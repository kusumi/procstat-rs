#[macro_use]
extern crate lazy_static;

mod buffer;
mod container;
mod frame;
mod panel;
mod util;
mod window;

const VERSION: [i32; 3] = [0, 1, 7];

const PROCSTAT_HOME: &str = "PROCSTAT_HOME";

#[cfg(feature = "curses")]
mod curses;

#[cfg(feature = "curses")]
use curses as screen;

#[cfg(feature = "stdout")]
mod stdout;

#[cfg(feature = "stdout")]
use stdout as screen;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug)]
struct Opt {
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

impl Default for Opt {
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

fn get_version_string() -> String {
    format!("{}.{}.{}", VERSION[0], VERSION[1], VERSION[2])
}

fn print_version() {
    println!("{}", get_version_string());
}

fn usage(progname: &str, opts: &getopts::Options) {
    println!(
        "{}",
        opts.usage(&format!("usage: {progname} [<options>] <paths>"))
    );
    println!(
        "Commands:
  0 - Set current position to the first line of the buffer
  $ - Set current position to the last line of the buffer
  k|UP - Scroll upward
  j|DOWN - Scroll downward
  h|LEFT - Select next window
  l|RIGHT - Select previous window
  CTRL-b - Scroll one page upward
  CTRL-u - Scroll half page upward
  CTRL-f - Scroll one page downward
  CTRL-d - Scroll half page downward
  CTRL-l - Repaint whole screen"
    );
}

fn init_file_logger(progname: &str) -> Result<()> {
    let home = util::get_home_path();
    let name = format!(".{}.log", util::get_basename(progname)?);
    let f = match std::env::var(PROCSTAT_HOME) {
        Ok(v) => {
            if util::is_dir(&v) {
                util::join_path(&v, &name)
            } else {
                println!("{PROCSTAT_HOME} not a directory, using {home} instead");
                util::join_path(&home, &name)
            }
        }
        Err(_) => util::join_path(&home, &name),
    };
    Ok(simplelog::CombinedLogger::init(vec![
        simplelog::WriteLogger::new(
            simplelog::LevelFilter::Trace,
            simplelog::Config::default(),
            std::fs::File::create(f)?,
        ),
    ])?)
}

static mut INTERRUPTED: bool = false;

extern "C" fn sigint_handler(_: libc::c_int) {
    log::info!("{}: SIGINT", util::function!());
    unsafe {
        INTERRUPTED = true;
    }
}

fn is_interrupted() -> bool {
    unsafe { INTERRUPTED }
}

extern "C" fn atexit_handler() {
    log::info!("{}: atexit", util::function!());
    screen::cleanup_screen().unwrap();
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let progname = &args[0];

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

    let matches = match opts.parse(&args[1..]) {
        Ok(v) => v,
        Err(e) => {
            println!("{e}");
            usage(progname, &opts);
            std::process::exit(1);
        }
    };
    if matches.opt_present("v") {
        print_version();
        std::process::exit(1);
    }
    if matches.opt_present("h") {
        usage(progname, &opts);
        std::process::exit(1);
    }

    let mut opt = Opt {
        ..Default::default()
    };
    let mut layout = match matches.opt_str("c") {
        Some(v) => v.to_lowercase(),
        None => String::new(),
    };
    opt.fgcolor = match matches.opt_str("fg") {
        Some(v) => screen::string_to_color(&v),
        None => -1,
    };
    opt.bgcolor = match matches.opt_str("bg") {
        Some(v) => screen::string_to_color(&v),
        None => -1,
    };
    opt.sinterval = match matches.opt_str("t") {
        Some(v) => match v.parse::<u64>() {
            Ok(v) => v,
            Err(e) => {
                println!("{v}: {e}");
                std::process::exit(1);
            }
        },
        None => 1,
    };
    if matches.opt_present("m") {
        let x = opt.sinterval;
        opt.sinterval = x / 1000;
        opt.minterval = x % 1000;
    }
    opt.showlnum = matches.opt_present("n");
    opt.foldline = matches.opt_present("f");
    opt.blinkline = !matches.opt_present("noblink");
    opt.rotatecol = matches.opt_present("r");
    opt.usedelay = matches.opt_present("usedelay");
    opt.debug = matches.opt_present("debug");

    if matches.free.is_empty() {
        usage(progname, &opts);
        std::process::exit(1);
    }

    let args = matches.free;
    if layout.is_empty() {
        layout = "1".repeat(args.len());
        assert!(!layout.is_empty());
    }
    for x in layout.chars() {
        if ('1'..='9').contains(&x) {
            let v = if let Some(v) = x.to_digit(10) {
                match v.try_into() {
                    Ok(v) => v,
                    Err(e) => {
                        println!("invalid value {v}: {e}");
                        std::process::exit(1);
                    }
                }
            } else {
                println!("invalid layout {layout}");
                std::process::exit(1);
            };
            opt.layout.push(v);
        } else if ('a'..='f').contains(&x) {
            let v = if let Some(v) = x.to_digit(16) {
                match v.try_into() {
                    Ok(v) => v,
                    Err(e) => {
                        println!("invalid value {v}: {e}");
                        std::process::exit(1);
                    }
                }
            } else {
                println!("invalid layout {layout}");
                std::process::exit(1);
            };
            opt.layout.push(v);
        } else {
            opt.layout.push(0);
        }
    }

    if opt.debug {
        if let Err(e) = init_file_logger(progname) {
            println!("{e}");
            std::process::exit(1);
        }
        log::info!("{opt:?}");
    }

    let attr = match screen::init_screen(opt.fgcolor, opt.bgcolor) {
        Ok(v) => v,
        Err(e) => {
            println!("{e}");
            std::process::exit(1);
        }
    };
    log::info!("{}: {:?}", util::function!(), attr);

    unsafe {
        libc::atexit(atexit_handler);
        libc::signal(libc::SIGINT, sigint_handler as usize);
    }

    let co = match container::Container::new(&args, attr, &opt) {
        Ok(v) => v,
        Err(e) => {
            println!("{e}");
            std::process::exit(1);
        }
    };
    let pair = std::sync::Arc::new((std::sync::Mutex::new(co), std::sync::Condvar::new()));
    let mut thrv = container::thread_create(&pair, &opt);
    loop {
        // XXX Do something outside of co.lock(), otherwise this loop never
        // releases the mutex, and as a result window threads get blocked.
        let x = screen::read_incoming();
        let (co, cv) = &*pair;
        let mut co = co.lock().unwrap();
        if let Err(e) = co.parse_event(x, cv, &opt) {
            println!("{e}");
            co.set_interrupted();
            cv.notify_all();
            break;
        }
        if is_interrupted() {
            co.set_interrupted();
            cv.notify_all();
            break;
        }
    }
    container::thread_join(&mut thrv);

    log::info!(
        "{}: {:?} exit",
        util::function!(),
        std::thread::current().id()
    );
}
