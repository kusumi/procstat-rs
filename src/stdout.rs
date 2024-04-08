use crate::util;
use crate::Result;
#[cfg(feature = "stdout")]
use std::io::Write;

lazy_static! {
    static ref MTX: std::sync::Mutex<i32> = std::sync::Mutex::new(0);
}

pub(crate) const KBD_ERR: i32 = 0xDEAD;
const KBD_BASE: u32 = 0xBEEF;
pub(crate) const KBD_UP: u32 = KBD_BASE;
pub(crate) const KBD_DOWN: u32 = KBD_BASE + 1;
pub(crate) const KBD_LEFT: u32 = KBD_BASE + 2;
pub(crate) const KBD_RIGHT: u32 = KBD_BASE + 3;
pub(crate) const KBD_RESIZE: u32 = KBD_BASE + 4;

pub(crate) fn kbd_ctrl(x: u32) -> u32 {
    x & 0x1F
}

#[derive(Debug, Default)]
pub(crate) struct Attr {
    lines: usize,
    cols: usize,
}

impl Attr {
    pub(crate) fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

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
    let mut attr = Attr::new();
    update_terminal_size(&mut attr)?;
    Ok(attr)
}

pub(crate) fn cleanup_screen() -> Result<()> {
    Ok(())
}

pub(crate) fn read_incoming() -> i32 {
    std::thread::sleep(std::time::Duration::from_secs(1));
    KBD_ERR
}

pub(crate) fn clear_terminal() -> Result<()> {
    Ok(())
}

pub(crate) fn flash_terminal() {}

pub(crate) fn alloc_screen(ylen: usize, xlen: usize, ypos: usize, xpos: usize) -> Result<Screen> {
    let _mtx = MTX.lock()?;
    let scr = Screen::new(ylen, xlen, ypos, xpos);
    println!(
        "{}: {:?} {} {} {} {}",
        stringify!(alloc_screen),
        scr,
        ylen,
        xlen,
        ypos,
        xpos
    );
    Ok(scr)
}

impl Screen {
    pub(crate) fn new(ylen: usize, xlen: usize, ypos: usize, xpos: usize) -> Self {
        let scr = Self {};
        println!(
            "{}: {:?} {} {} {} {}",
            stringify!(new),
            scr,
            ylen,
            xlen,
            ypos,
            xpos
        );
        scr
    }

    pub(crate) fn delete(&mut self) -> Result<()> {
        let _mtx = MTX.lock()?;
        println!("{}: {:?}", stringify!(delete), self);
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
            "{}: {:?} {} {} {} {} \"{}\"",
            stringify!(print),
            self,
            y,
            x,
            standout,
            standout_attr,
            s
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
