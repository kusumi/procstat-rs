use crate::UserData;

#[cfg(feature = "curses")]
use crate::curses as screen;

#[cfg(feature = "stdout")]
use crate::stdout as screen;

#[derive(Debug, Default)]
pub struct Panel {
    scr: screen::Screen,
    ylen: usize,
    xlen: usize,
    ypos: usize,
    xpos: usize,
}

pub trait PanelImpl {
    fn new(ylen: usize, xlen: usize, ypos: usize, xpos: usize, dat: &UserData) -> Self;
    fn get_ylen(&self) -> usize;
    fn get_xlen(&self) -> usize;
    fn get_ypos(&self) -> usize;
    fn get_xpos(&self) -> usize;
    fn set_title(&mut self, _s: &str) {}
    fn set_focus(&mut self, _t: bool) {}
    fn refresh(&mut self);
    fn erase(&mut self);
    fn resize(&mut self, ylen: usize, xlen: usize, ypos: usize, xpos: usize, dat: &mut UserData);
    fn print(&self, y: usize, x: usize, standout: bool, standout_attr: u32, s: &str);
}

impl Drop for Panel {
    fn drop(&mut self) {
        screen::delete_screen(&mut self.scr).unwrap();
    }
}

impl PanelImpl for Panel {
    fn new(ylen: usize, xlen: usize, ypos: usize, xpos: usize, dat: &UserData) -> Self {
        let scr = screen::alloc_screen(ylen, xlen, ypos, xpos).unwrap();
        let mut panel = Panel {
            scr,
            ylen,
            xlen,
            ypos,
            xpos,
        };
        screen::bkgd_screen(&mut panel.scr, dat).unwrap();
        panel
    }

    fn get_ylen(&self) -> usize {
        self.ylen
    }

    fn get_xlen(&self) -> usize {
        self.xlen
    }

    fn get_ypos(&self) -> usize {
        self.ypos
    }

    fn get_xpos(&self) -> usize {
        self.xpos
    }

    fn refresh(&mut self) {
        screen::refresh_screen(&mut self.scr).unwrap();
    }

    fn erase(&mut self) {
        screen::erase_screen(&mut self.scr).unwrap();
    }

    fn resize(&mut self, ylen: usize, xlen: usize, ypos: usize, xpos: usize, dat: &mut UserData) {
        self._resize(ylen, xlen, ypos, xpos, dat);
        self.refresh();
    }

    fn print(&self, y: usize, x: usize, standout: bool, standout_attr: u32, s: &str) {
        screen::print_screen(&self.scr, y, x, standout, standout_attr, s).unwrap();
    }
}

impl Panel {
    fn _resize(&mut self, ylen: usize, xlen: usize, ypos: usize, xpos: usize, dat: &mut UserData) {
        self.ylen = ylen;
        self.xlen = xlen;
        self.ypos = ypos;
        self.xpos = xpos;
        screen::resize_screen(&mut self.scr, self.ylen, self.xlen, dat).unwrap();
        screen::move_screen(&mut self.scr, self.ypos, self.xpos).unwrap();
    }
}
