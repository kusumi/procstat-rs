use crate::panel::PanelImpl;

#[derive(Debug)]
pub struct Window {
    panel: crate::panel::Panel,
    frame: crate::frame::Frame,
    buffer: crate::buffer::Buffer,
    offset: usize,
}

impl Window {
    pub fn new(
        ylen: usize,
        xlen: usize,
        ypos: usize,
        xpos: usize,
        dat: &crate::UserData,
    ) -> Window {
        let mut w = Window {
            frame: crate::frame::Frame::new(ylen, xlen, ypos, xpos, dat),
            panel: crate::panel::Panel::new(ylen - 2, xlen - 2, ypos + 1, xpos + 1, dat),
            buffer: crate::buffer::Buffer::new(),
            offset: 0,
        };
        w.frame.refresh();
        w.panel.refresh();
        w
    }

    pub fn is_dead(&mut self) -> bool {
        self.buffer.is_dead()
    }

    pub fn resize(
        &mut self,
        ylen: usize,
        xlen: usize,
        ypos: usize,
        xpos: usize,
        dat: &mut crate::UserData,
    ) {
        self.frame.resize(ylen, xlen, ypos, xpos, dat);
        self.panel
            .resize(ylen - 2, xlen - 2, ypos + 1, xpos + 1, dat);
        self.offset = 0;
    }

    pub fn attach_buffer(&mut self, f: &str) -> Result<(), std::io::Error> {
        assert!(self.buffer.get_path().as_str() == "");
        self.buffer.set_reader(f)?; // do this first
        self.frame.set_title(f);
        self.panel.set_title(f);
        log::info!("window={:?} path={}", self, self.buffer.get_path());
        Ok(())
    }

    // used by watch
    #[allow(dead_code)]
    pub fn update_buffer(&mut self) {
        self.buffer.update().unwrap();
        log::info!("window={:?} path={}", self, self.buffer.get_path());
    }

    pub fn focus(&mut self, t: bool) {
        self.frame.set_focus(t);
        self.panel.set_focus(t);
    }

    pub fn goto_head(&mut self) {
        self.offset = 0;
    }

    pub fn goto_tail(&mut self) {
        self.offset = self.buffer.get_max_line();
    }

    pub fn goto_current(&mut self, d: isize) {
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

    pub fn repaint(&mut self, showlnum: bool, foldline: bool, blinkline: bool, standout_attr: u32) {
        if self.is_dead() {
            return;
        }

        let mut pos = 0;
        let mut y = 0;
        let mut s = String::new();
        let mut standout = false;
        let offset = self.offset;
        let xlen = self.panel.get_xlen();

        self.buffer.block_till_ready();
        self.panel.erase();

        loop {
            match self
                .buffer
                .readline(&mut pos, &mut s, &mut standout, showlnum, blinkline)
            {
                Ok(()) => (),
                Err(_) => break,
            }
            // C++ / Go version with fine grained lock checks ylen/xlen/offset mismatch here

            if pos < offset {
                continue;
            }
            // XXX expecting s to only contain ascii
            if !foldline && s.len() > xlen {
                s = s.get(0..xlen).unwrap().to_string();
            }
            self.panel.print(y, 0, standout, standout_attr, &s);
            if !foldline {
                y += 1;
            } else {
                y += s.len() / xlen;
                if s.len() % xlen != 0 {
                    y += 1;
                }
            }
        }

        self.panel.refresh();
        self.buffer.clear().unwrap();
        self.buffer.signal_blocked();
    }

    pub fn signal(&mut self) {
        // XXX should be pthread_cond_signal equivalent
    }

    pub fn timedwait(&mut self, msec: u64) {
        // XXX should be pthread_cond_timedwait equivalent
        std::thread::sleep(std::time::Duration::from_millis(msec));
    }
}
