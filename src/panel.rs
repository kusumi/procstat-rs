use crate::Result;
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
    fn new(ylen: usize, xlen: usize, ypos: usize, xpos: usize, dat: &UserData) -> Result<Self>
    where
        Self: Sized;
    fn get_ylen(&self) -> usize;
    fn get_xlen(&self) -> usize;
    fn get_ypos(&self) -> usize;
    fn get_xpos(&self) -> usize;
    fn set_title(&mut self, _s: &str) -> Result<()> {
        Ok(())
    }
    fn set_focus(&mut self, _t: bool) -> Result<()> {
        Ok(())
    }
    fn refresh(&mut self) -> Result<()>;
    fn erase(&mut self) -> Result<()>;
    fn resize(
        &mut self,
        ylen: usize,
        xlen: usize,
        ypos: usize,
        xpos: usize,
        dat: &mut UserData,
    ) -> Result<()>;
    fn print(&self, y: usize, x: usize, standout: bool, standout_attr: u32, s: &str) -> Result<()>;
}

impl Drop for Panel {
    fn drop(&mut self) {
        self.scr.delete().unwrap();
    }
}

impl PanelImpl for Panel {
    fn new(ylen: usize, xlen: usize, ypos: usize, xpos: usize, dat: &UserData) -> Result<Self>
    where
        Self: Sized,
    {
        let scr = screen::alloc_screen(ylen, xlen, ypos, xpos)?;
        let mut panel = Panel {
            scr,
            ..Default::default()
        };
        panel.update_size(ylen, xlen, ypos, xpos);
        panel.scr.bkgd(dat)?;
        Ok(panel)
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
        self.update_size(ylen, xlen, ypos, xpos);
        self.refresh()
    }

    fn print(&self, y: usize, x: usize, standout: bool, standout_attr: u32, s: &str) -> Result<()> {
        self.scr.print(y, x, standout, standout_attr, s)
    }
}

impl Panel {
    fn update_size(&mut self, ylen: usize, xlen: usize, ypos: usize, xpos: usize) {
        self.ylen = ylen;
        self.xlen = xlen;
        self.ypos = ypos;
        self.xpos = xpos;
    }
}
