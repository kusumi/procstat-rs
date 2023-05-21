#[macro_use]
extern crate lazy_static;

mod buffer;
mod container;
mod frame;
mod panel;
mod util;
mod window;

// curses
#[cfg(feature = "curses")]
mod curses;

#[cfg(feature = "curses")]
use curses as screen;

// stdout
#[cfg(feature = "stdout")]
mod stdout;

#[cfg(feature = "stdout")]
use stdout as screen;

lazy_static! {
    pub static ref MTX: std::sync::Mutex<i32> = std::sync::Mutex::new(0);
}

const VERSION: [i32; 3] = [0, 1, 1];

#[derive(Debug)]
struct UserOption {
    layout: Vec<usize>,
    sinterval: u64,
    minterval: u64,
    fgcolor: i16,
    bgcolor: i16,
    showlnum: bool,
    foldline: bool,
    rotatecol: bool,
    blinkline: bool,
    usedelay: bool,
    debug: bool,
}

impl Default for UserOption {
    fn default() -> Self {
        Self {
            layout: Vec::new(),
            sinterval: 1,
            minterval: 0,
            fgcolor: -1,
            bgcolor: -1,
            showlnum: false,
            foldline: false,
            rotatecol: false,
            blinkline: true,
            usedelay: false,
            debug: false,
        }
    }
}

#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct UserData {
    opt: UserOption,
    term: screen::Terminal,
    color_attr: u32,
    standout_attr: u32,
}

fn get_version_string() -> String {
    format!("{}.{}.{}", VERSION[0], VERSION[1], VERSION[2])
}

fn print_version() {
    println!("{}", get_version_string());
}

fn usage(progname: &str, opts: getopts::Options) {
    let brief = format!("usage: {} [<options>] <paths>", progname);
    print!("{}", opts.usage(&brief));
}

fn init_log(f: &str) {
    simplelog::CombinedLogger::init(vec![simplelog::WriteLogger::new(
        simplelog::LevelFilter::Info,
        simplelog::Config::default(),
        std::fs::File::create(f).unwrap(),
    )])
    .unwrap();
    assert!(std::path::Path::new(&f).is_file());
}

static mut INTERRUPTED: bool = false;

extern "C" fn sigint_handler(_: libc::c_int) {
    log::info!("{}: SIGINT", stringify!(sigint_handler));
    unsafe {
        INTERRUPTED = true;
    }
}

extern "C" fn atexit_handler() {
    log::info!("{}: atexit", stringify!(atexit_handler));
    screen::cleanup_screen().unwrap();
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let progname = args[0].clone();

    let mut opts = getopts::Options::new();
    opts.optopt(
        "c",
        "",
        "Set column layout. \
            e.g. \"-c 123\" to make 3 columns with 1,2,3 windows for each",
        "STRING",
    );
    opts.optopt(
        "",
        "fg",
        "Set foreground color. Available colors are \
            \"black\", \"blue\", \"cyan\", \"green\", \"magenta\", \"red\", \"white\", \"yellow\".",
        "STRING",
    );
    opts.optopt(
        "",
        "bg",
        "Set background color. Available colors are \
            \"black\", \"blue\", \"cyan\", \"green\", \"magenta\", \"red\", \"white\", \"yellow\".",
        "STRING",
    );
    opts.optopt(
        "t",
        "",
        "Set refresh interval in second. Default is 1. \
            e.g. \"-t 5\" to refresh screen every 5 seconds",
        "STRING",
    );
    opts.optflag(
        "m",
        "",
        "Take refresh interval as milli second. \
            e.g. \"-t 500 -m\" to refresh screen every 500 milli seconds",
    );
    opts.optflag("n", "", "Show line number");
    opts.optflag("f", "", "Fold lines when longer than window width");
    opts.optflag("r", "", "Rotate column layout");
    opts.optflag("", "noblink", "Disable blink");
    opts.optflag(
        "",
        "usedelay",
        "Add random delay time before each window starts",
    );
    opts.optflag("", "debug", "Enable debug log");
    opts.optflag("v", "version", "Print version and exit");
    opts.optflag("h", "help", "print this help menu");

    let matches = opts.parse(&args[1..]).unwrap();
    if matches.opt_present("v") {
        print_version();
        std::process::exit(1);
    }
    if matches.opt_present("h") {
        usage(&progname, opts);
        std::process::exit(1);
    }

    let mut dat = UserData {
        ..Default::default()
    };
    let mut layout = match matches.opt_str("c") {
        Some(v) => v.to_lowercase(),
        None => "".to_string(),
    };
    dat.opt.fgcolor = match matches.opt_str("fg") {
        Some(v) => screen::string_to_color(&v),
        None => -1,
    };
    dat.opt.bgcolor = match matches.opt_str("bg") {
        Some(v) => screen::string_to_color(&v),
        None => -1,
    };
    dat.opt.sinterval = match matches.opt_str("t") {
        Some(v) => v.parse::<u64>().unwrap(),
        None => 1,
    };
    if matches.opt_present("m") {
        let x = dat.opt.sinterval;
        dat.opt.sinterval = x / 1000;
        dat.opt.minterval = x % 1000;
    }
    dat.opt.showlnum = matches.opt_present("n");
    dat.opt.foldline = matches.opt_present("f");
    dat.opt.blinkline = !matches.opt_present("noblink");
    dat.opt.rotatecol = matches.opt_present("r");
    dat.opt.usedelay = matches.opt_present("usedelay");
    dat.opt.debug = matches.opt_present("debug");

    if matches.free.is_empty() {
        usage(&progname, opts);
        std::process::exit(1);
    }

    let args = matches.free;
    if layout.is_empty() {
        layout = "1".repeat(args.len());
        assert!(!layout.is_empty());
    }
    for x in layout.chars() {
        if ('1'..='9').contains(&x) {
            dat.opt.layout.push(x as usize - '0' as usize);
        } else if ('a'..='f').contains(&x) {
            dat.opt.layout.push(x as usize - 'a' as usize + 10);
        } else {
            dat.opt.layout.push(0);
        }
    }

    if dat.opt.debug {
        let home = dirs::home_dir()
            .unwrap()
            .into_os_string()
            .into_string()
            .unwrap();
        init_log(&util::join_path(&home, ".procstat.log"));
    }

    match screen::init_screen(&mut dat) {
        Ok(_) if dat.opt.debug => log::info!("{:?}", dat.term),
        Ok(_) => (),
        Err(e) => panic!("{}", e),
    };

    if dat.opt.debug {
        log::info!("{:?}", dat);
    }

    unsafe {
        libc::atexit(atexit_handler);
        libc::signal(libc::SIGINT, sigint_handler as usize);
    }

    let mut co = container::Container::new(args, &mut dat);
    unsafe {
        co.thread_create(&mut dat);
        while !INTERRUPTED {
            co.parse_event(screen::read_incoming(), &mut dat);
        }
        co.thread_join();
    }
    log::info!("{}: exit", stringify!(main));
}
