use crate::panel;
use crate::panel::PanelImpl;
use crate::UserData;

#[cfg(feature = "curses")]
use crate::curses as screen;

#[cfg(feature = "stdout")]
use crate::stdout as screen;

#[derive(Debug, Default)]
pub struct Frame {
    scr: screen::Screen,
    ylen: usize,
    xlen: usize,
    ypos: usize,
    xpos: usize,
    title: String,
    focus: bool,
}

impl Drop for Frame {
    fn drop(&mut self) {
        screen::delete_screen(&mut self.scr).unwrap();
    }
}

impl panel::PanelImpl for Frame {
    fn new(ylen: usize, xlen: usize, ypos: usize, xpos: usize, dat: &UserData) -> Self {
        let scr = screen::alloc_screen(ylen, xlen, ypos, xpos).unwrap();
        let mut frame = Frame {
            scr,
            ylen,
            xlen,
            ypos,
            xpos,
            title: String::from(""),
            focus: false,
        };
        screen::bkgd_screen(&mut frame.scr, dat).unwrap();
        screen::box_screen(&mut frame.scr).unwrap();
        frame
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

    fn set_title(&mut self, s: &str) {
        self.title = String::from(s);
        self.print_title();
    }

    fn set_focus(&mut self, t: bool) {
        self.focus = t;
        self.print_title();
    }

    fn refresh(&mut self) {
        screen::refresh_screen(&mut self.scr).unwrap();
    }

    fn erase(&mut self) {
        screen::erase_screen(&mut self.scr).unwrap();
    }

    fn resize(&mut self, ylen: usize, xlen: usize, ypos: usize, xpos: usize, dat: &mut UserData) {
        self._resize(ylen, xlen, ypos, xpos, dat);
        screen::box_screen(&mut self.scr).unwrap();
        self.print_title();
    }

    fn print(&self, y: usize, x: usize, standout: bool, standout_attr: u32, s: &str) {
        screen::print_screen(&self.scr, y, x, standout, standout_attr, s).unwrap();
    }
}

impl Frame {
    fn _resize(&mut self, ylen: usize, xlen: usize, ypos: usize, xpos: usize, dat: &mut UserData) {
        self.ylen = ylen;
        self.xlen = xlen;
        self.ypos = ypos;
        self.xpos = xpos;
        screen::resize_screen(&mut self.scr, self.ylen, self.xlen, dat).unwrap();
        screen::move_screen(&mut self.scr, self.ypos, self.xpos).unwrap();
    }

    fn print_title(&mut self) {
        self.print(0, 1, self.focus, 0, &self.title);
        self.refresh();
    }
}
