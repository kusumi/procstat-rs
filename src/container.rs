#[derive(Debug, Default)]
pub struct Container {
    v: Vec<std::sync::Arc<std::sync::Mutex<crate::window::Window>>>,
    t: Vec<std::thread::JoinHandle<()>>,
    i: usize,
}

/*
impl Drop for Container {
    fn drop(&mut self) {
        // delete_watch();
    }
}
*/

impl Container {
    pub fn new(args: Vec<String>, dat: &mut crate::UserData) -> Self {
        let mut co = Container {
            v: Vec::new(),
            t: Vec::new(),
            i: 0,
        };
        co.build_window(dat);
        for (i, f) in args.iter().enumerate() {
            if !crate::util::is_regular_file(f) {
                log::info!("No such regular file {}", f);
                continue;
            }
            if i < co.v.len() {
                match co.v[i].lock().unwrap().attach_buffer(f) {
                    Ok(_) => (),
                    Err(e) => log::info!("{}", e),
                }
            }
        }
        co.v[co.i].lock().unwrap().focus(true);
        co
    }

    fn goto_next_window(&mut self) {
        let vlen = self.v.len();
        let begi = self.i;
        self.v[begi].lock().unwrap().focus(false);
        loop {
            let w = &mut self.v[self.i].lock().unwrap();
            self.i += 1;
            if self.i == vlen {
                self.i = 0;
            }
            if !w.is_dead() || self.i == begi {
                break;
            }
        }
        self.v[begi].lock().unwrap().focus(true);
    }

    fn goto_prev_window(&mut self) {
        let vlen = self.v.len();
        let begi = self.i;
        self.v[begi].lock().unwrap().focus(false);
        loop {
            let w = &mut self.v[self.i].lock().unwrap();
            self.i -= 1;
            if self.i == 0 {
                self.i = vlen - 1;
            }
            if !w.is_dead() || self.i == begi {
                break;
            }
        }
        self.v[begi].lock().unwrap().focus(true);
    }

    fn build_window(&mut self, dat: &mut crate::UserData) {
        if !dat.opt.rotatecol {
            self.build_window_xy(dat);
        } else {
            self.build_window_yx(dat);
        }
    }

    fn build_window_xy(&mut self, dat: &mut crate::UserData) {
        let mut seq = 0;
        let xx = dat.term.get_terminal_cols();
        let yy = dat.term.get_terminal_lines();
        let x = dat.opt.layout.len();
        let xq = xx / x;
        let xr = xx % x;

        for i in 0..x {
            let xpos = xq * i;
            let mut xlen = xq;
            if i == x - 1 {
                xlen += xr;
            }
            let mut y = dat.opt.layout[i];
            if y == 0 {
                y = 1; // ignore invalid
            }
            let yq = yy / y;
            let yr = yy % y;

            for j in 0..y {
                let ypos = yq * j;
                let mut ylen = yq;
                if j == y - 1 {
                    ylen += yr;
                }
                self.alloc_window(seq, ylen, xlen, ypos, xpos, dat);
                seq += 1;
            }
        }
    }

    fn build_window_yx(&mut self, dat: &mut crate::UserData) {
        let mut seq = 0;
        let yy = dat.term.get_terminal_lines();
        let xx = dat.term.get_terminal_cols();
        let y = dat.opt.layout.len();
        let yq = yy / y;
        let yr = yy % y;

        for i in 0..y {
            let ypos = yq * i;
            let mut ylen = yq;
            if i == y - 1 {
                ylen += yr;
            }
            let mut x = dat.opt.layout[i];
            if x == 0 {
                x = 1; // ignore invalid
            }
            let xq = xx / x;
            let xr = xx % x;

            for j in 0..x {
                let xpos = xq * j;
                let mut xlen = xq;
                if j == x - 1 {
                    xlen += xr;
                }
                self.alloc_window(seq, ylen, xlen, ypos, xpos, dat);
                seq += 1;
            }
        }
    }

    fn alloc_window(
        &mut self,
        seq: usize,
        ylen: usize,
        xlen: usize,
        ypos: usize,
        xpos: usize,
        dat: &mut crate::UserData,
    ) {
        if let Some(p) = self.v.get_mut(seq) {
            let w = &mut p.lock().unwrap();
            w.resize(ylen, xlen, ypos, xpos, dat);
            w.signal();
        } else {
            self.v.push(std::sync::Arc::new(std::sync::Mutex::new(
                crate::window::Window::new(ylen, xlen, ypos, xpos, dat),
            )));
        }
    }

    // XXX self.v[self.i].lock() blocks when window threads are alive, why ?
    pub fn parse_event(&mut self, x: isize, dat: &mut crate::UserData) {
        if x == crate::KBD_ERR {
        } else if x == crate::KBD_RESIZE || x == crate::kbd_ctrl('l' as isize) {
            crate::clear_terminal().unwrap();
            self.build_window(dat);
        } else if x == 'h' as isize || x == crate::KBD_LEFT {
            self.goto_prev_window();
        } else if x == 'l' as isize || x == crate::KBD_RIGHT {
            self.goto_next_window();
        } else if x == '0' as isize {
            let w = &mut self.v[self.i].lock().unwrap();
            w.goto_head();
            w.signal();
        } else if x == '$' as isize {
            let w = &mut self.v[self.i].lock().unwrap();
            w.goto_tail();
            w.signal();
        } else if x == 'k' as isize || x == crate::KBD_UP {
            let w = &mut self.v[self.i].lock().unwrap();
            w.goto_current(-1);
            w.signal();
        } else if x == 'j' as isize || x == crate::KBD_DOWN {
            let w = &mut self.v[self.i].lock().unwrap();
            w.goto_current(1);
            w.signal();
        } else if x == crate::kbd_ctrl('B' as isize) {
            let w = &mut self.v[self.i].lock().unwrap();
            w.goto_current(-(dat.term.get_terminal_lines() as isize));
            w.signal();
        } else if x == crate::kbd_ctrl('U' as isize) {
            let w = &mut self.v[self.i].lock().unwrap();
            w.goto_current(-(dat.term.get_terminal_lines() as isize) / 2);
            w.signal();
        } else if x == crate::kbd_ctrl('F' as isize) {
            let w = &mut self.v[self.i].lock().unwrap();
            w.goto_current(dat.term.get_terminal_lines() as isize);
            w.signal();
        } else if x == crate::kbd_ctrl('D' as isize) {
            let w = &mut self.v[self.i].lock().unwrap();
            w.goto_current(dat.term.get_terminal_lines() as isize / 2);
            w.signal();
        } else {
            let w = &mut self.v[self.i].lock().unwrap();
            w.signal();
        }
    }

    // create window threads
    pub fn thread_create(&mut self, dat: &mut crate::UserData) {
        for v in self.v.iter_mut() {
            let sinterval = dat.opt.sinterval;
            let minterval = dat.opt.minterval;
            let showlnum = dat.opt.showlnum;
            let foldline = dat.opt.foldline;
            let blinkline = dat.opt.blinkline;
            let usedelay = dat.opt.usedelay;
            let standout_attr = dat.standout_attr;
            let cv = v.clone();

            let t = std::thread::spawn(move || {
                let mut t = 0;
                if usedelay {
                    let r: u64 = rand::prelude::random();
                    if sinterval != 0 {
                        t = r % 1000;
                    } else {
                        t = r % (1000 * 1000);
                    }
                }
                if t != 0 {
                    let w = &mut cv.lock().unwrap();
                    w.repaint(showlnum, foldline, blinkline, standout_attr);
                    w.timedwait(t);
                }
                unsafe {
                    while !crate::INTERRUPTED {
                        let w = &mut cv.lock().unwrap();
                        w.repaint(showlnum, foldline, blinkline, standout_attr);
                        w.timedwait(sinterval * 1000 + minterval);
                        // w is unlocked here once, correct ?
                    }
                }
            });
            log::info!("{}: {:?}", stringify!(thread_create), t.thread().id());
            self.t.push(t);
        }
    }

    // join window threads
    pub fn thread_join(&mut self) {
        // https://stackoverflow.com/questions/68966949/unable-to-join-threads-from-joinhandles-stored-in-a-vector-rust
        while let Some(t) = self.t.pop() {
            log::info!("{}: {:?}", stringify!(thread_join), t.thread().id());
            t.join().unwrap();
        }
    }
}
