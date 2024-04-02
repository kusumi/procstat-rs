use crate::util;
use crate::window;
use crate::Opt;
use crate::Result;

#[cfg(feature = "curses")]
use crate::curses as screen;

#[cfg(feature = "stdout")]
use crate::stdout as screen;

#[derive(Debug)]
pub(crate) struct Container {
    v: Vec<window::Window>,
    biv: Vec<usize>,
    wih: std::collections::HashMap<i32, usize>,
    ci: usize,
    attr: screen::Attr,
    inotify: inotify::Inotify,
    is_interrupted: bool,
}

impl Default for Container {
    fn default() -> Self {
        Self {
            v: Vec::new(),
            biv: Vec::new(),
            wih: std::collections::HashMap::new(),
            ci: 0,
            attr: screen::Attr::new(),
            inotify: inotify::Inotify::init().unwrap(),
            is_interrupted: false,
        }
    }
}

impl Container {
    pub(crate) fn new(args: Vec<String>, attr: screen::Attr, opt: &Opt) -> Result<Self> {
        let mut co = Self {
            attr,
            ..Default::default()
        };
        co.init(args, opt)?;
        Ok(co)
    }

    fn init(&mut self, args: Vec<String>, opt: &Opt) -> Result<()> {
        self.build_window(opt)?;
        for (i, f) in args.iter().enumerate() {
            if !util::is_regular_file(f) {
                log::info!("{}: No such regular file {}", stringify!(init), f);
                continue;
            }
            if i < self.v.len() {
                if let Err(e) = self.v[i].attach_buffer(f) {
                    log::info!("{}: {}", stringify!(init), e);
                    break;
                }
                self.biv.push(i);
                let wd = self
                    .inotify
                    .watches()
                    .add(util::get_abspath(f)?, inotify::WatchMask::MODIFY)?;
                self.wih.insert(wd.get_watch_descriptor_id(), i);
            }
        }
        self.v[self.ci].focus(true, self.attr.get_standout_attr())
    }

    fn goto_next_window(&mut self) -> Result<()> {
        let a = self.attr.get_standout_attr();
        self.v[self.ci].focus(false, 0)?;
        for (i, &idx) in self.biv.iter().enumerate() {
            if idx == self.ci {
                if idx == self.biv[self.biv.len() - 1] {
                    self.ci = self.biv[0];
                } else {
                    self.ci = self.biv[i + 1];
                }
                return self.v[self.ci].focus(true, a);
            }
        }
        if !self.biv.is_empty() {
            self.ci = self.biv[0];
            return self.v[self.ci].focus(true, a);
        }
        Ok(())
    }

    fn goto_prev_window(&mut self) -> Result<()> {
        let a = self.attr.get_standout_attr();
        self.v[self.ci].focus(false, 0)?;
        for (i, &idx) in self.biv.iter().enumerate() {
            if idx == self.ci {
                if idx == self.biv[0] {
                    self.ci = self.biv[self.biv.len() - 1];
                } else {
                    self.ci = self.biv[i - 1];
                }
                return self.v[self.ci].focus(true, a);
            }
        }
        if !self.biv.is_empty() {
            self.ci = self.biv[self.biv.len() - 1];
            return self.v[self.ci].focus(true, a);
        }
        Ok(())
    }

    fn build_window(&mut self, opt: &Opt) -> Result<()> {
        if !opt.rotatecol {
            self.build_window_xy(opt)
        } else {
            self.build_window_yx(opt)
        }
    }

    fn build_window_xy(&mut self, opt: &Opt) -> Result<()> {
        let mut seq = 0;
        let xx = self.attr.get_terminal_cols();
        let yy = self.attr.get_terminal_lines();
        let x = opt.layout.len();
        let xq = xx / x;
        let xr = xx % x;

        for i in 0..x {
            let xpos = xq * i;
            let mut xlen = xq;
            if i == x - 1 {
                xlen += xr;
            }
            let mut y = opt.layout[i];
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
                self.alloc_window(seq, ylen, xlen, ypos, xpos)?;
                seq += 1;
            }
        }
        Ok(())
    }

    fn build_window_yx(&mut self, opt: &Opt) -> Result<()> {
        let mut seq = 0;
        let yy = self.attr.get_terminal_lines();
        let xx = self.attr.get_terminal_cols();
        let y = opt.layout.len();
        let yq = yy / y;
        let yr = yy % y;

        for i in 0..y {
            let ypos = yq * i;
            let mut ylen = yq;
            if i == y - 1 {
                ylen += yr;
            }
            let mut x = opt.layout[i];
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
                self.alloc_window(seq, ylen, xlen, ypos, xpos)?;
                seq += 1;
            }
        }
        Ok(())
    }

    fn alloc_window(
        &mut self,
        seq: usize,
        ylen: usize,
        xlen: usize,
        ypos: usize,
        xpos: usize,
    ) -> Result<()> {
        if self.v.get(seq).is_some() {
            self.v[seq].resize(ylen, xlen, ypos, xpos, &mut self.attr)?;
            //self.v[seq].signal(); // XXX
        } else {
            self.v
                .push(window::Window::new(ylen, xlen, ypos, xpos, &self.attr)?);
        }
        log::info!(
            "{}: seq {}, len({}, {}), pos({}, {})",
            stringify!(alloc_window),
            seq,
            ylen,
            xlen,
            ypos,
            xpos
        );
        Ok(())
    }

    pub(crate) fn parse_event(&mut self, x: i32, cv: &std::sync::Condvar, opt: &Opt) -> Result<()> {
        if x == screen::KBD_ERR {
            //log::info!("{}: KBD_ERR", stringify!(parse_event));
            return Ok(());
        }
        let x = u32::try_from(x).unwrap();
        if x == screen::KBD_RESIZE || x == screen::kbd_ctrl(u32::from('l')) {
            screen::update_terminal_size(&mut self.attr)?;
            screen::clear_terminal()?;
            self.build_window(opt)?;
        } else if x == u32::from('h') || x == screen::KBD_LEFT {
            self.goto_prev_window()?;
        } else if x == u32::from('l') || x == screen::KBD_RIGHT {
            self.goto_next_window()?;
        } else if x == u32::from('0') {
            let w = &mut self.v[self.ci];
            w.goto_head();
            cv.notify_all();
        } else if x == u32::from('$') {
            let w = &mut self.v[self.ci];
            w.goto_tail();
            cv.notify_all();
        } else if x == u32::from('k') || x == screen::KBD_UP {
            let w = &mut self.v[self.ci];
            w.goto_current(-1);
            cv.notify_all();
        } else if x == u32::from('j') || x == screen::KBD_DOWN {
            let w = &mut self.v[self.ci];
            w.goto_current(1);
            cv.notify_all();
        } else if x == screen::kbd_ctrl(u32::from('B')) {
            let w = &mut self.v[self.ci];
            w.goto_current(-isize::try_from(self.attr.get_terminal_lines()).unwrap());
            cv.notify_all();
        } else if x == screen::kbd_ctrl(u32::from('U')) {
            let w = &mut self.v[self.ci];
            w.goto_current(-isize::try_from(self.attr.get_terminal_lines()).unwrap() / 2);
            cv.notify_all();
        } else if x == screen::kbd_ctrl(u32::from('F')) {
            let w = &mut self.v[self.ci];
            w.goto_current(isize::try_from(self.attr.get_terminal_lines()).unwrap());
            cv.notify_all();
        } else if x == screen::kbd_ctrl(u32::from('D')) {
            let w = &mut self.v[self.ci];
            w.goto_current(isize::try_from(self.attr.get_terminal_lines()).unwrap() / 2);
            cv.notify_all();
        } else {
            cv.notify_all();
        }
        Ok(())
    }

    pub(crate) fn set_interrupted(&mut self) {
        self.is_interrupted = true;
        log::info!("{}: interrupted", stringify!(set_interrupted));
    }

    pub(crate) fn is_interrupted(&self) -> bool {
        self.is_interrupted
    }
}

fn thread_create_watch(
    pair: &std::sync::Arc<(std::sync::Mutex<Container>, std::sync::Condvar)>,
) -> std::thread::JoinHandle<()> {
    let pair = std::sync::Arc::clone(pair);
    std::thread::spawn(move || {
        let tid = std::thread::current().id();
        let (co, cv) = &*pair;
        loop {
            let mut buf = [0; 1024];
            let mut co = co.lock().unwrap();
            match co.inotify.read_events(&mut buf) {
                Ok(v) => {
                    for event in v {
                        match co.wih.get(&event.wd.get_watch_descriptor_id()) {
                            Some(&i) => {
                                log::info!("{:?} watch {} {:?}", tid, i, co.wih);
                                co.v[i].update_buffer().unwrap();
                            }
                            _ => {
                                log::info!("{:?} {:?}", tid, event);
                                return;
                            }
                        }
                    }
                    screen::flash_terminal().unwrap();
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => (),
                Err(e) => {
                    log::info!("{:?} {}", tid, e);
                    return;
                }
            };
            let ret = cv
                .wait_timeout(co, std::time::Duration::from_secs(1))
                .unwrap();
            if ret.0.is_interrupted() {
                log::info!("{:?} watch interrupted", tid);
                break;
            }
        }
    })
}

fn thread_create_window(
    pair: &std::sync::Arc<(std::sync::Mutex<Container>, std::sync::Condvar)>,
    opt: &Opt,
) -> Vec<std::thread::JoinHandle<()>> {
    let mut thrv = Vec::new();
    let (co, _) = &**pair;
    let n = co.lock().unwrap().v.len();

    for i in 0..n {
        let sinterval = opt.sinterval;
        let minterval = opt.minterval;
        let showlnum = opt.showlnum;
        let foldline = opt.foldline;
        let blinkline = opt.blinkline;
        let usedelay = opt.usedelay;
        let pair = std::sync::Arc::clone(pair);
        thrv.push(std::thread::spawn(move || {
            let tid = std::thread::current().id();
            let (co, cv) = &*pair;
            let t = sinterval * 1000 + minterval;
            let mut d = t;
            if usedelay {
                let r: u64 = rand::prelude::random();
                d = r % 1000;
            }
            loop {
                let mut co = co.lock().unwrap();
                let a = co.attr.get_standout_attr();
                co.v[i].repaint(showlnum, foldline, blinkline, a).unwrap();
                let ret = cv
                    .wait_timeout(co, std::time::Duration::from_millis(d))
                    .unwrap();
                if ret.0.is_interrupted() {
                    log::info!("{:?} window interrupted", tid);
                    break;
                }
                d = t;
            }
        }));
    }
    thrv
}

// XXX Threads lock the entire container, whereas in C++ / Go they only
// lock shared resource, i.e. terminal size and buffers.
pub(crate) fn thread_create(
    pair: &std::sync::Arc<(std::sync::Mutex<Container>, std::sync::Condvar)>,
    opt: &Opt,
) -> Vec<std::thread::JoinHandle<()>> {
    let mut thrv = Vec::new();
    thrv.push(thread_create_watch(pair));
    thrv.extend(thread_create_window(pair, opt));
    for thr in thrv.iter() {
        log::info!("{}: {:?}", stringify!(thread_create), thr.thread().id());
    }
    thrv
}

pub(crate) fn thread_join(thrv: &mut Vec<std::thread::JoinHandle<()>>) {
    while let Some(thr) = thrv.pop() {
        log::info!("{}: {:?}", stringify!(thread_join), thr.thread().id());
        thr.join().unwrap();
    }
}
