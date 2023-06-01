procstat-rs ([v0.1.2](https://github.com/kusumi/procstat-rs/releases/tag/v0.1.2))
========

## About

+ Ncurses based file monitor.

+ Rust version of [https://github.com/kusumi/procstat-go](https://github.com/kusumi/procstat-go).

![procstat](https://a.fsdn.com/con/app/proj/procfsv/screenshots/318601.jpg/max/max/1)

## Requirements

Recent version of Rust

## Build

    $ make

or

    $ gmake

## Usage

    $ ./target/release/procstat-rs
    usage: ./target/release/procstat-rs [<options>] <paths>
    
    Options:
        -c STRING           Set column layout. e.g. "-c 123" to make 3 columns
                            with 1,2,3 windows for each
            --fg STRING     Set foreground color. Available colors are "black",
                            "blue", "cyan", "green", "magenta", "red", "white",
                            "yellow".
            --bg STRING     Set background color. Available colors are "black",
                            "blue", "cyan", "green", "magenta", "red", "white",
                            "yellow".
        -t STRING           Set refresh interval in second. Default is 1. e.g. "-t
                            5" to refresh screen every 5 seconds
        -m                  Take refresh interval as milli second. e.g. "-t 500
                            -m" to refresh screen every 500 milli seconds
        -n                  Show line number
        -f                  Fold lines when longer than window width
        -r                  Rotate column layout
            --noblink       Disable blink
            --usedelay      Add random delay time before each window starts
            --debug         Enable debug log
        -v, --version       Print version and exit
        -h, --help          print this help menu

## Bug

+ A watcher thread is unimplemented in this Rust version.

+ Commands are currently unsupported. This is actually a bug, see XXX comment in src/container.rs.

## Resource

+ Upstream [https://sourceforge.net/projects/procfsv](https://sourceforge.net/projects/procfsv)

+ Repository [https://github.com/kusumi/procstat-rs](https://github.com/kusumi/procstat-rs)
