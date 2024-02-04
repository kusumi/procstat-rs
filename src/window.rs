use crate::buffer;
use crate::frame;
use crate::panel;
use crate::panel::PanelImpl;
use crate::Result;

#[cfg(feature = "curses")]
use crate::curses as screen;

#[cfg(feature = "stdout")]
use crate::stdout as screen;

#[derive(Debug)]
pub(crate) struct Window {
    panel: panel::Panel,
    frame: frame::Frame,
    buffer: buffer::Buffer,
    offset: usize,
}

impl Window {
    pub(crate) fn new(
        ylen: usize,
        xlen: usize,
        ypos: usize,
        xpos: usize,
        attr: &screen::Attr,
    ) -> Result<Window> {
        let mut w = Window {
            frame: frame::Frame::new(ylen, xlen, ypos, xpos, attr)?,
            panel: panel::Panel::new(ylen - 2, xlen - 2, ypos + 1, xpos + 1, attr)?,
            buffer: buffer::Buffer::new()?,
            offset: 0,
        };
        w.frame.refresh()?;
        w.panel.refresh()?;
        Ok(w)
    }

    pub(crate) fn is_dead(&mut self) -> bool {
        self.buffer.is_dead()
    }

    pub(crate) fn resize(
        &mut self,
        ylen: usize,
        xlen: usize,
        ypos: usize,
        xpos: usize,
        attr: &mut screen::Attr,
    ) -> Result<()> {
        self.frame.resize(ylen, xlen, ypos, xpos, attr)?;
        self.panel
            .resize(ylen - 2, xlen - 2, ypos + 1, xpos + 1, attr)?;
        self.offset = 0;
        Ok(())
    }

    pub(crate) fn attach_buffer(&mut self, f: &str) -> Result<()> {
        self.buffer.init(f)?; // still had no path set at this point
        self.frame.set_title(f)?;
        self.panel.set_title(f)?;
        log::info!(
            "{}: {:?} {:?}",
            stringify!(attach_buffer),
            self.panel,
            self.frame,
        );
        Ok(())
    }

    pub(crate) fn update_buffer(&mut self) -> std::io::Result<()> {
        self.buffer.update()?;
        log::info!(
            "{}: {:?} {:?}",
            stringify!(update_buffer),
            self.panel,
            self.frame,
        );
        Ok(())
    }

    pub(crate) fn focus(&mut self, t: bool, standout_attr: u32) -> Result<()> {
        self.frame.set_focus(t, standout_attr)?;
        self.panel.set_focus(t, standout_attr)
    }

    pub(crate) fn goto_head(&mut self) {
        self.offset = 0;
    }

    pub(crate) fn goto_tail(&mut self) {
        self.offset = self.buffer.get_max_line();
    }

    pub(crate) fn goto_current(&mut self, d: isize) {
        self.offset = if d < 0 {
            if self.offset < d.unsigned_abs() {
                0
            } else {
                self.offset - d.unsigned_abs()
            }
        } else if self.offset + d as usize > self.buffer.get_max_line() {
            self.buffer.get_max_line()
        } else {
            self.offset + d as usize
        }
    }

    pub(crate) fn repaint(
        &mut self,
        showlnum: bool,
        foldline: bool,
        blinkline: bool,
        standout_attr: u32,
    ) -> Result<()> {
        if self.is_dead() {
            return Ok(());
        }

        let mut pos = 0;
        let mut y = 0;
        let mut s = String::new();
        let mut standout = false;
        let offset = self.offset;
        let xlen = self.panel.get_xlen();

        self.buffer.block_till_ready();
        self.panel.erase()?;

        loop {
            if self
                .buffer
                .readline(&mut pos, &mut s, &mut standout, showlnum, blinkline)
                .is_err()
            {
                break;
            }
            // C++ / Go version with fine grained lock checks ylen/xlen/offset mismatch here

            if pos < offset {
                continue;
            }
            // XXX expecting s to only contain ascii
            if !foldline && s.len() > xlen {
                s = s.get(0..xlen).ok_or_else(|| xlen.to_string())?.to_string();
            }
            self.panel.print(y, 0, standout, standout_attr, &s)?;
            if !foldline {
                y += 1;
            } else {
                y += s.len() / xlen;
                if s.len() % xlen != 0 {
                    y += 1;
                }
            }
        }

        self.panel.refresh()?;
        self.buffer.clear()?;
        self.buffer.signal_blocked();
        Ok(())
    }
}
