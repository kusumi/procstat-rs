use crate::util;
use crate::Result;
use crate::UserData;
#[cfg(feature = "curses")]

lazy_static! {
    static ref MTX: std::sync::Mutex<i32> = std::sync::Mutex::new(0);
}

// taken from /usr/include/curses.h
// XXX ncurses::KEY_xxx ?
pub const KBD_ERR: isize = -1;
pub const KBD_UP: isize = 0o403;
pub const KBD_DOWN: isize = 0o402;
pub const KBD_LEFT: isize = 0o404;
pub const KBD_RIGHT: isize = 0o405;
pub const KBD_RESIZE: isize = 0o632;

// taken from /usr/include/curses.h
// XXX ncurses::COLOR_xxx ?
const COLOR_BLACK: i16 = 0;
const COLOR_RED: i16 = 1;
const COLOR_GREEN: i16 = 2;
const COLOR_YELLOW: i16 = 3;
const COLOR_BLUE: i16 = 4;
const COLOR_MAGENTA: i16 = 5;
const COLOR_CYAN: i16 = 6;
const COLOR_WHITE: i16 = 7;

pub fn kbd_ctrl(x: isize) -> isize {
    x & 0x1F
}

#[derive(Debug, Default)]
pub struct Terminal {
    lines: usize,
    cols: usize,
}

impl Terminal {
    pub fn get_terminal_lines(&self) -> usize {
        self.lines
    }

    pub fn get_terminal_cols(&self) -> usize {
        self.cols
    }
}

#[derive(Debug)]
pub struct Screen {
    win: ncurses::WINDOW,
}

unsafe impl Send for Screen {}

impl Default for Screen {
    fn default() -> Self {
        Self {
            win: ncurses::newwin(0, 0, 0, 0),
        }
    }
}

fn update_terminal_size(dat: &mut UserData) -> Result<()> {
    let _mtx = MTX.lock()?;
    let mut y = 0;
    let mut x = 0;
    ncurses::getmaxyx(ncurses::stdscr(), &mut y, &mut x);
    dat.term.lines = y as usize;
    dat.term.cols = x as usize;
    Ok(())
}

pub fn string_to_color(arg: &str) -> i16 {
    match arg {
        "black" => COLOR_BLACK,
        "red" => COLOR_RED,
        "green" => COLOR_GREEN,
        "yellow" => COLOR_YELLOW,
        "blue" => COLOR_BLUE,
        "magenta" => COLOR_MAGENTA,
        "cyan" => COLOR_CYAN,
        "white" => COLOR_WHITE,
        _ => -1,
    }
}

pub fn init_screen(dat: &mut UserData) -> Result<()> {
    ncurses::initscr();
    ncurses::keypad(ncurses::stdscr(), true);
    ncurses::noecho();
    ncurses::cbreak();
    ncurses::curs_set(ncurses::CURSOR_VISIBILITY::CURSOR_INVISIBLE).ok_or(util::error())?;
    ncurses::wtimeout(ncurses::stdscr(), 500);
    clear_terminal()?;
    update_terminal_size(dat)?;

    if ncurses::has_colors() {
        ncurses::start_color();
        ncurses::use_default_colors();
        ncurses::init_pair(1, dat.opt.fgcolor, dat.opt.bgcolor);
        dat.color_attr = ncurses::COLOR_PAIR(1);
    }

    dat.standout_attr = match std::env::var("TERM") {
        Ok(v) if v == "screen" => ncurses::A_REVERSE(),
        _ => ncurses::A_STANDOUT(),
    };
    Ok(())
}

pub fn cleanup_screen() -> Result<()> {
    ncurses::curs_set(ncurses::CURSOR_VISIBILITY::CURSOR_VISIBLE).ok_or(util::error())?;
    ncurses::endwin();
    Ok(())
}

pub fn read_incoming() -> isize {
    ncurses::wgetch(ncurses::stdscr()) as isize
}

pub fn clear_terminal() -> Result<()> {
    let _mtx = MTX.lock()?;
    ncurses::wclear(ncurses::stdscr());
    ncurses::wrefresh(ncurses::stdscr());
    Ok(())
}

// used by watch
#[allow(dead_code)]
pub fn flash_terminal() -> Result<()> {
    ncurses::flash();
    Ok(())
}

pub fn alloc_screen(ylen: usize, xlen: usize, ypos: usize, xpos: usize) -> Result<Screen> {
    let _mtx = MTX.lock()?;
    let scr = Screen::new(ylen, xlen, ypos, xpos);
    ncurses::scrollok(scr.win, false);
    ncurses::idlok(scr.win, false);
    ncurses::keypad(scr.win, true);
    Ok(scr)
}

impl Screen {
    pub fn new(ylen: usize, xlen: usize, ypos: usize, xpos: usize) -> Self {
        Self {
            win: ncurses::newwin(ylen as i32, xlen as i32, ypos as i32, xpos as i32),
        }
    }

    pub fn delete(&mut self) -> Result<()> {
        let _mtx = MTX.lock()?;
        ncurses::delwin(self.win);
        Ok(())
    }

    pub fn print(
        &self,
        y: usize,
        x: usize,
        standout: bool,
        standout_attr: u32, // ncurses::attr_t
        s: &str,
    ) -> Result<()> {
        let _mtx = MTX.lock()?;
        let attr = if standout {
            if standout_attr == 0 {
                // XXX used by Frame::print_title
                ncurses::A_STANDOUT()
            } else {
                standout_attr
            }
        } else {
            ncurses::A_NORMAL()
        };
        ncurses::wattron(self.win, attr);
        ncurses::mvwprintw(self.win, y as i32, x as i32, s);
        ncurses::wattroff(self.win, attr);
        Ok(())
    }

    pub fn refresh(&mut self) -> Result<()> {
        let _mtx = MTX.lock()?;
        ncurses::wrefresh(self.win);
        Ok(())
    }

    pub fn erase(&mut self) -> Result<()> {
        let _mtx = MTX.lock()?;
        ncurses::werase(self.win);
        Ok(())
    }

    pub fn resize(&mut self, ylen: usize, xlen: usize, dat: &mut UserData) -> Result<()> {
        let _mtx = MTX.lock()?;
        ncurses::wresize(self.win, ylen as i32, xlen as i32);
        update_terminal_size(dat)?;
        Ok(())
    }

    pub fn r#move(&mut self, ypos: usize, xpos: usize) -> Result<()> {
        let _mtx = MTX.lock()?;
        ncurses::mvwin(self.win, ypos as i32, xpos as i32);
        Ok(())
    }

    pub fn r#box(&mut self) -> Result<()> {
        let _mtx = MTX.lock()?;
        ncurses::wborder(
            self.win,
            ncurses::ACS_VLINE(),
            ncurses::ACS_VLINE(),
            ncurses::ACS_HLINE(),
            ncurses::ACS_HLINE(),
            ncurses::ACS_ULCORNER(),
            ncurses::ACS_URCORNER(),
            ncurses::ACS_LLCORNER(),
            ncurses::ACS_LRCORNER(),
        );
        Ok(())
    }

    pub fn bkgd(&mut self, dat: &UserData) -> Result<()> {
        let _mtx = MTX.lock()?;
        if dat.color_attr != ncurses::A_NORMAL() {
            ncurses::wbkgd(self.win, dat.color_attr | ' ' as u32);
        }
        Ok(())
    }
}
