use crate::UserData;
#[cfg(feature = "curses")]
use crate::MTX;

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

#[derive(Debug, Default)]
pub struct Screen {
    win: Option<ncurses::WINDOW>,
}

unsafe impl Send for Screen {}

impl Screen {
    pub fn new(ylen: usize, xlen: usize, ypos: usize, xpos: usize) -> Self {
        Self {
            win: Some(ncurses::newwin(
                ylen as i32,
                xlen as i32,
                ypos as i32,
                xpos as i32,
            )),
        }
    }
}

fn update_terminal_size(dat: &mut UserData) -> Result<(), Box<dyn std::error::Error>> {
    let _mtx = MTX.lock().unwrap();
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

pub fn init_screen(dat: &mut UserData) -> Result<(), Box<dyn std::error::Error>> {
    ncurses::initscr();
    ncurses::keypad(ncurses::stdscr(), true);
    ncurses::noecho();
    ncurses::cbreak();
    if ncurses::curs_set(ncurses::CURSOR_VISIBILITY::CURSOR_INVISIBLE).is_some() {}
    ncurses::wtimeout(ncurses::stdscr(), 500);
    clear_terminal()?;
    update_terminal_size(dat)?;

    if ncurses::has_colors() {
        ncurses::start_color();
        ncurses::use_default_colors();
        ncurses::init_pair(1, dat.opt.fgcolor, dat.opt.bgcolor);
        dat.color_attr = ncurses::COLOR_PAIR(1);
    }

    match std::env::var("TERM") {
        Ok(v) if v == "screen" => dat.standout_attr = ncurses::A_REVERSE(),
        _ => dat.standout_attr = ncurses::A_STANDOUT(),
    }
    Ok(())
}

pub fn cleanup_screen() -> Result<(), Box<dyn std::error::Error>> {
    if ncurses::curs_set(ncurses::CURSOR_VISIBILITY::CURSOR_VISIBLE).is_some() {}
    ncurses::endwin();
    Ok(())
}

pub fn read_incoming() -> isize {
    ncurses::wgetch(ncurses::stdscr()) as isize
}

pub fn clear_terminal() -> Result<(), Box<dyn std::error::Error>> {
    let _mtx = MTX.lock().unwrap();
    ncurses::wclear(ncurses::stdscr());
    ncurses::wrefresh(ncurses::stdscr());
    Ok(())
}

// used by watch
#[allow(dead_code)]
pub fn flash_terminal() -> Result<(), Box<dyn std::error::Error>> {
    ncurses::flash();
    Ok(())
}

pub fn alloc_screen(
    ylen: usize,
    xlen: usize,
    ypos: usize,
    xpos: usize,
) -> Result<Screen, Box<dyn std::error::Error>> {
    let _mtx = MTX.lock().unwrap();
    let scr = Screen::new(ylen, xlen, ypos, xpos);
    let win = scr.win.unwrap();
    ncurses::scrollok(win, false);
    ncurses::idlok(win, false);
    ncurses::keypad(win, true);
    Ok(scr)
}

pub fn delete_screen(scr: &mut Screen) -> Result<(), Box<dyn std::error::Error>> {
    let _mtx = MTX.lock().unwrap();
    let win = scr.win.unwrap();
    ncurses::delwin(win);
    Ok(())
}

pub fn print_screen(
    scr: &Screen,
    y: usize,
    x: usize,
    standout: bool,
    standout_attr: ncurses::attr_t,
    s: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let _mtx = MTX.lock().unwrap();
    let win = scr.win.unwrap();
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
    ncurses::wattron(win, attr);
    ncurses::mvwprintw(win, y as i32, x as i32, s);
    ncurses::wattroff(win, attr);
    Ok(())
}

pub fn refresh_screen(scr: &mut Screen) -> Result<(), Box<dyn std::error::Error>> {
    let _mtx = MTX.lock().unwrap();
    let win = scr.win.unwrap();
    ncurses::wrefresh(win);
    Ok(())
}

pub fn erase_screen(scr: &mut Screen) -> Result<(), Box<dyn std::error::Error>> {
    let _mtx = MTX.lock().unwrap();
    let win = scr.win.unwrap();
    ncurses::werase(win);
    Ok(())
}

pub fn resize_screen(
    scr: &mut Screen,
    ylen: usize,
    xlen: usize,
    dat: &mut UserData,
) -> Result<(), Box<dyn std::error::Error>> {
    let _mtx = MTX.lock().unwrap();
    let win = scr.win.unwrap();
    ncurses::wresize(win, ylen as i32, xlen as i32);
    update_terminal_size(dat)?;
    Ok(())
}

pub fn move_screen(
    scr: &mut Screen,
    ypos: usize,
    xpos: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let _mtx = MTX.lock().unwrap();
    let win = scr.win.unwrap();
    ncurses::mvwin(win, ypos as i32, xpos as i32);
    Ok(())
}

pub fn box_screen(scr: &mut Screen) -> Result<(), Box<dyn std::error::Error>> {
    let _mtx = MTX.lock().unwrap();
    let win = scr.win.unwrap();
    ncurses::wborder(
        win,
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

pub fn bkgd_screen(scr: &mut Screen, dat: &UserData) -> Result<(), Box<dyn std::error::Error>> {
    let _mtx = MTX.lock().unwrap();
    let win = scr.win.unwrap();
    if dat.color_attr != ncurses::A_NORMAL() {
        ncurses::wbkgd(win, dat.color_attr | ' ' as u32);
    }
    Ok(())
}
