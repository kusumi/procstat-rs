use crate::panel;
use crate::panel::PanelImpl;
use crate::Result;
use crate::UserData;

#[cfg(feature = "curses")]
use crate::curses as screen;

#[cfg(feature = "stdout")]
use crate::stdout as screen;

#[derive(Debug, Default)]
pub struct Frame {
    scr: screen::Screen,
    title: String,
    focus: bool,
    ylen: usize,
    xlen: usize,
    ypos: usize,
    xpos: usize,
}

impl Drop for Frame {
    fn drop(&mut self) {
        self.scr.delete().unwrap();
    }
}

impl panel::PanelImpl for Frame {
    fn new(ylen: usize, xlen: usize, ypos: usize, xpos: usize, dat: &UserData) -> Result<Self>
    where
        Self: Sized,
    {
        let scr = screen::alloc_screen(ylen, xlen, ypos, xpos)?;
        let mut frame = Frame {
            scr,
            title: "".to_string(),
            focus: false,
            ..Default::default()
        };
        frame.update_size(ylen, xlen, ypos, xpos);
        frame.scr.bkgd(dat)?;
        frame.scr.r#box()?;
        Ok(frame)
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

    fn set_title(&mut self, s: &str) -> Result<()> {
        self.title = s.to_string();
        self.print_title()
    }

    fn set_focus(&mut self, t: bool) -> Result<()> {
        self.focus = t;
        self.print_title()
    }

    fn refresh(&mut self) -> Result<()> {
        self.scr.refresh()
    }

    fn erase(&mut self) -> Result<()> {
        self.scr.erase()
    }

    fn resize(
        &mut self,
        ylen: usize,
        xlen: usize,
        ypos: usize,
        xpos: usize,
        dat: &mut UserData,
    ) -> Result<()> {
        self.scr.resize(self.ylen, self.xlen, dat)?;
        self.scr.r#move(self.ypos, self.xpos)?;
        self.scr.r#box()?;
        self.update_size(ylen, xlen, ypos, xpos);
        self.print_title()
    }

    fn print(&self, y: usize, x: usize, standout: bool, standout_attr: u32, s: &str) -> Result<()> {
        self.scr.print(y, x, standout, standout_attr, s)
    }
}

impl Frame {
    fn update_size(&mut self, ylen: usize, xlen: usize, ypos: usize, xpos: usize) {
        self.ylen = ylen;
        self.xlen = xlen;
        self.ypos = ypos;
        self.xpos = xpos;
    }

    fn print_title(&mut self) -> Result<()> {
        self.print(0, 1, self.focus, 0, &self.title)?;
        self.refresh()
    }
}
