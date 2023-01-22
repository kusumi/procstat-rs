use crate::panel::PanelImpl;

#[derive(Debug, Default)]
pub struct Frame {
    scr: crate::Screen,
    ylen: usize,
    xlen: usize,
    ypos: usize,
    xpos: usize,
    title: String,
    focus: bool,
}

impl Drop for Frame {
    fn drop(&mut self) {
        crate::delete_screen(&mut self.scr).unwrap();
    }
}

impl crate::panel::PanelImpl for Frame {
    fn new(ylen: usize, xlen: usize, ypos: usize, xpos: usize, dat: &crate::UserData) -> Self {
        let scr = crate::alloc_screen(ylen, xlen, ypos, xpos).unwrap();
        let mut frame = Frame {
            scr,
            ylen,
            xlen,
            ypos,
            xpos,
            title: String::from(""),
            focus: false,
        };
        crate::bkgd_screen(&mut frame.scr, dat).unwrap();
        crate::box_screen(&mut frame.scr).unwrap();
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
        crate::refresh_screen(&mut self.scr).unwrap();
    }

    fn erase(&mut self) {
        crate::erase_screen(&mut self.scr).unwrap();
    }

    fn resize(
        &mut self,
        ylen: usize,
        xlen: usize,
        ypos: usize,
        xpos: usize,
        dat: &mut crate::UserData,
    ) {
        self._resize(ylen, xlen, ypos, xpos, dat);
        crate::box_screen(&mut self.scr).unwrap();
        self.print_title();
    }

    fn print(&self, y: usize, x: usize, standout: bool, standout_attr: u32, s: &str) {
        crate::print_screen(&self.scr, y, x, standout, standout_attr, s).unwrap();
    }
}

impl Frame {
    fn _resize(
        &mut self,
        ylen: usize,
        xlen: usize,
        ypos: usize,
        xpos: usize,
        dat: &mut crate::UserData,
    ) {
        self.ylen = ylen;
        self.xlen = xlen;
        self.ypos = ypos;
        self.xpos = xpos;
        crate::resize_screen(&mut self.scr, self.ylen, self.xlen, dat).unwrap();
        crate::move_screen(&mut self.scr, self.ypos, self.xpos).unwrap();
    }

    fn print_title(&mut self) {
        self.print(0, 1, self.focus, 0, &self.title);
        self.refresh();
    }
}
