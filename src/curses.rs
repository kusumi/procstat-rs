use crate::util;
use crate::Result;
#[cfg(feature = "curses")]

lazy_static! {
    static ref MTX: std::sync::Mutex<i32> = std::sync::Mutex::new(0);
}

// taken from /usr/include/curses.h
// XXX ncurses::KEY_xxx ?
pub(crate) const KEY_ERR: i32 = -1;
pub(crate) const KEY_UP: u32 = 0o403;
pub(crate) const KEY_DOWN: u32 = 0o402;
pub(crate) const KEY_LEFT: u32 = 0o404;
pub(crate) const KEY_RIGHT: u32 = 0o405;
pub(crate) const KEY_RESIZE: u32 = 0o632;

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

pub(crate) fn key_ctrl(x: u32) -> u32 {
    x & 0x1F
}

#[derive(Debug, Default)]
pub(crate) struct Attr {
    lines: usize,
    cols: usize,
    color_attr: u32,
    standout_attr: u32,
}

impl Attr {
    pub(crate) fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub(crate) fn get_terminal_lines(&self) -> usize {
        self.lines
    }

    pub(crate) fn get_terminal_cols(&self) -> usize {
        self.cols
    }

    pub(crate) fn get_color_attr(&self) -> u32 {
        self.color_attr
    }

    pub(crate) fn get_standout_attr(&self) -> u32 {
        self.standout_attr
    }
}

#[derive(Debug)]
pub(crate) struct Screen {
    win: ncurses::WINDOW,
}

unsafe impl Send for Screen {}

pub(crate) fn update_terminal_size(attr: &mut Attr) -> Result<()> {
    let _mtx = MTX.lock()?;
    let mut y = 0;
    let mut x = 0;
    ncurses::getmaxyx(ncurses::stdscr(), &mut y, &mut x);
    attr.lines = y.try_into()?;
    attr.cols = x.try_into()?;
    log::info!("{}: {:?}", util::function!(), attr);
    Ok(())
}

pub(crate) fn string_to_color(arg: &str) -> i16 {
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

pub(crate) fn init_screen(fgcolor: i16, bgcolor: i16) -> Result<Attr> {
    ncurses::initscr();
    ncurses::keypad(ncurses::stdscr(), true);
    ncurses::noecho();
    ncurses::cbreak();
    ncurses::curs_set(ncurses::CURSOR_VISIBILITY::CURSOR_INVISIBLE).ok_or_else(util::error)?;
    ncurses::wtimeout(ncurses::stdscr(), 500);
    clear_terminal()?;

    let mut attr = Attr::new();
    update_terminal_size(&mut attr)?;

    if ncurses::has_colors() {
        ncurses::start_color();
        ncurses::use_default_colors();
        ncurses::init_pair(1, fgcolor, bgcolor);
        attr.color_attr = ncurses::COLOR_PAIR(1);
    }

    attr.standout_attr = match std::env::var("TERM") {
        Ok(v) if v == "screen" => ncurses::A_REVERSE(),
        _ => ncurses::A_STANDOUT(),
    };
    Ok(attr)
}

pub(crate) fn cleanup_screen() -> Result<()> {
    ncurses::curs_set(ncurses::CURSOR_VISIBILITY::CURSOR_VISIBLE).ok_or_else(util::error)?;
    ncurses::endwin();
    Ok(())
}

pub(crate) fn read_incoming() -> i32 {
    ncurses::wgetch(ncurses::stdscr())
}

pub(crate) fn clear_terminal() -> Result<()> {
    let _mtx = MTX.lock()?;
    ncurses::wclear(ncurses::stdscr());
    ncurses::wrefresh(ncurses::stdscr());
    Ok(())
}

pub(crate) fn flash_terminal() {
    ncurses::flash();
}

pub(crate) fn alloc_screen(ylen: usize, xlen: usize, ypos: usize, xpos: usize) -> Result<Screen> {
    let _mtx = MTX.lock()?;
    let scr = Screen::new(ylen, xlen, ypos, xpos);
    ncurses::scrollok(scr.win, false);
    ncurses::idlok(scr.win, false);
    ncurses::keypad(scr.win, true);
    Ok(scr)
}

impl Screen {
    pub(crate) fn new(ylen: usize, xlen: usize, ypos: usize, xpos: usize) -> Self {
        Self {
            win: ncurses::newwin(
                ylen.try_into().unwrap(),
                xlen.try_into().unwrap(),
                ypos.try_into().unwrap(),
                xpos.try_into().unwrap(),
            ),
        }
    }

    pub(crate) fn delete(&mut self) -> Result<()> {
        let _mtx = MTX.lock()?;
        ncurses::delwin(self.win);
        Ok(())
    }

    pub(crate) fn print(
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
                ncurses::A_NORMAL()
            } else {
                standout_attr
            }
        } else {
            ncurses::A_NORMAL()
        };
        ncurses::wattron(self.win, attr);
        ncurses::mvwprintw(self.win, y.try_into()?, x.try_into()?, s);
        ncurses::wattroff(self.win, attr);
        Ok(())
    }

    pub(crate) fn refresh(&mut self) -> Result<()> {
        let _mtx = MTX.lock()?;
        ncurses::wrefresh(self.win);
        Ok(())
    }

    pub(crate) fn erase(&mut self) -> Result<()> {
        let _mtx = MTX.lock()?;
        ncurses::werase(self.win);
        Ok(())
    }

    pub(crate) fn resize(&mut self, ylen: usize, xlen: usize) -> Result<()> {
        let _mtx = MTX.lock()?;
        ncurses::wresize(self.win, ylen.try_into()?, xlen.try_into()?);
        Ok(())
    }

    pub(crate) fn r#move(&mut self, ypos: usize, xpos: usize) -> Result<()> {
        let _mtx = MTX.lock()?;
        ncurses::mvwin(self.win, ypos.try_into()?, xpos.try_into()?);
        Ok(())
    }

    pub(crate) fn r#box(&mut self) -> Result<()> {
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

    pub(crate) fn bkgd(&mut self, color_attr: u32) -> Result<()> {
        let _mtx = MTX.lock()?;
        if color_attr != ncurses::A_NORMAL() {
            ncurses::wbkgd(self.win, color_attr | u32::from(' '));
        }
        Ok(())
    }
}
