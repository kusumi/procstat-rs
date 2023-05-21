use crate::UserData;
#[cfg(feature = "stdout")]
use crate::MTX;
use std::io::Write;

pub const KBD_ERR: isize = 0xDEAD;
pub const KBD_UP: isize = KBD_ERR + 1;
pub const KBD_DOWN: isize = KBD_ERR + 2;
pub const KBD_LEFT: isize = KBD_ERR + 3;
pub const KBD_RIGHT: isize = KBD_ERR + 4;
pub const KBD_RESIZE: isize = KBD_ERR + 5;

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
        assert!(self.lines != 0);
        self.lines
    }

    pub fn get_terminal_cols(&self) -> usize {
        assert!(self.cols != 0);
        self.cols
    }
}

#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct Screen {
    ylen: usize,
    xlen: usize,
    ypos: usize,
    xpos: usize,
}

impl Screen {
    pub fn new(ylen: usize, xlen: usize, ypos: usize, xpos: usize) -> Self {
        Self {
            ylen,
            xlen,
            ypos,
            xpos,
        }
    }
}

fn update_terminal_size(dat: &mut UserData) -> Result<(), Box<dyn std::error::Error>> {
    let _mtx = MTX.lock().unwrap();
    if let Some((w, h)) = term_size::dimensions() {
        dat.term.lines = h;
        dat.term.cols = w;
        Ok(())
    } else {
        Err(Box::new(std::io::Error::from(std::io::ErrorKind::Other)))
    }
}

pub fn string_to_color(_arg: &str) -> i16 {
    -1
}

pub fn init_screen(dat: &mut UserData) -> Result<(), Box<dyn std::error::Error>> {
    update_terminal_size(dat)
}

pub fn cleanup_screen() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

pub fn read_incoming() -> isize {
    std::thread::sleep(std::time::Duration::from_secs(1));
    KBD_ERR
}

pub fn clear_terminal() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

// used by watch
#[allow(dead_code)]
pub fn flash_terminal() -> Result<(), Box<dyn std::error::Error>> {
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
    println!("Allocate {:?}", scr);
    Ok(scr)
}

pub fn delete_screen(scr: &mut Screen) -> Result<(), Box<dyn std::error::Error>> {
    let _mtx = MTX.lock().unwrap();
    println!("Delete {:?}", scr);
    Ok(())
}

pub fn print_screen(
    scr: &Screen,
    y: usize,
    x: usize,
    standout: bool,
    _standout_attr: u32,
    s: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let _mtx = MTX.lock().unwrap();
    println!("Print {:?}: {} {} {} \"{}\"", scr, y, x, standout, s);
    Ok(())
}

pub fn refresh_screen(_scr: &mut Screen) -> Result<(), Box<dyn std::error::Error>> {
    let _mtx = MTX.lock().unwrap();
    std::io::stdout().flush().unwrap();
    Ok(())
}

pub fn erase_screen(_scr: &mut Screen) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

pub fn resize_screen(
    _scr: &mut Screen,
    _ylen: usize,
    _xlen: usize,
    dat: &mut UserData,
) -> Result<(), Box<dyn std::error::Error>> {
    update_terminal_size(dat)
}

pub fn move_screen(
    _scr: &mut Screen,
    _ypos: usize,
    _xpos: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

pub fn box_screen(_scr: &mut Screen) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

pub fn bkgd_screen(_scr: &mut Screen, _dat: &UserData) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}
