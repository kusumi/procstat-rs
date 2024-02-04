use crate::util;
use crate::Result;
#[cfg(feature = "stdout")]
use std::io::Write;

lazy_static! {
    static ref MTX: std::sync::Mutex<i32> = std::sync::Mutex::new(0);
}

pub(crate) const KBD_ERR: isize = 0xDEAD;
pub(crate) const KBD_UP: isize = KBD_ERR + 1;
pub(crate) const KBD_DOWN: isize = KBD_ERR + 2;
pub(crate) const KBD_LEFT: isize = KBD_ERR + 3;
pub(crate) const KBD_RIGHT: isize = KBD_ERR + 4;
pub(crate) const KBD_RESIZE: isize = KBD_ERR + 5;

pub(crate) fn kbd_ctrl(x: isize) -> isize {
    x & 0x1F
}

#[derive(Debug, Default)]
pub(crate) struct Attr {
    lines: usize,
    cols: usize,
}

impl Attr {
    pub(crate) fn get_terminal_lines(&self) -> usize {
        assert!(self.lines != 0);
        self.lines
    }

    pub(crate) fn get_terminal_cols(&self) -> usize {
        assert!(self.cols != 0);
        self.cols
    }

    pub(crate) fn get_color_attr(&self) -> u32 {
        0
    }

    pub(crate) fn get_standout_attr(&self) -> u32 {
        0
    }
}

pub(crate) fn newattr() -> Attr {
    Attr {
        ..Default::default()
    }
}

#[derive(Debug, Default)]
pub(crate) struct Screen {}

pub(crate) fn update_terminal_size(attr: &mut Attr) -> Result<()> {
    let _mtx = MTX.lock()?;
    if let Some((w, h)) = term_size::dimensions() {
        attr.lines = h;
        attr.cols = w;
        log::info!("{}: {:?}", stringify!(update_terminal_size), attr);
        Ok(())
    } else {
        Err(Box::new(util::error()))
    }
}

pub(crate) fn string_to_color(_arg: &str) -> i16 {
    -1
}

pub(crate) fn init_screen(_fgcolor: i16, _bgcolor: i16) -> Result<Attr> {
    let mut attr = newattr();
    update_terminal_size(&mut attr)?;
    Ok(attr)
}

pub(crate) fn cleanup_screen() -> Result<()> {
    Ok(())
}

pub(crate) fn read_incoming() -> isize {
    std::thread::sleep(std::time::Duration::from_secs(1));
    KBD_ERR
}

pub(crate) fn clear_terminal() -> Result<()> {
    Ok(())
}

pub(crate) fn flash_terminal() -> Result<()> {
    Ok(())
}

pub(crate) fn alloc_screen(ylen: usize, xlen: usize, ypos: usize, xpos: usize) -> Result<Screen> {
    let _mtx = MTX.lock()?;
    let scr = Screen::new(ylen, xlen, ypos, xpos);
    println!("Allocate {:?}", scr);
    Ok(scr)
}

impl Screen {
    pub(crate) fn new(_ylen: usize, _xlen: usize, _ypos: usize, _xpos: usize) -> Self {
        Self {}
    }

    pub(crate) fn delete(&mut self) -> Result<()> {
        let _mtx = MTX.lock()?;
        println!("Delete {:?}", self);
        Ok(())
    }

    pub(crate) fn print(
        &self,
        y: usize,
        x: usize,
        standout: bool,
        standout_attr: u32,
        s: &str,
    ) -> Result<()> {
        let _mtx = MTX.lock()?;
        println!(
            "Print {:?}: {} {} {} {} \"{}\"",
            self, y, x, standout, standout_attr, s
        );
        Ok(())
    }

    pub(crate) fn refresh(&mut self) -> Result<()> {
        let _mtx = MTX.lock()?;
        std::io::stdout().flush()?;
        Ok(())
    }

    pub(crate) fn erase(&mut self) -> Result<()> {
        Ok(())
    }

    pub(crate) fn resize(&mut self, _ylen: usize, _xlen: usize) -> Result<()> {
        Ok(())
    }

    pub(crate) fn r#move(&mut self, _ypos: usize, _xpos: usize) -> Result<()> {
        Ok(())
    }

    pub(crate) fn r#box(&mut self) -> Result<()> {
        Ok(())
    }

    pub(crate) fn bkgd(&mut self, _color_attr: u32) -> Result<()> {
        Ok(())
    }
}
