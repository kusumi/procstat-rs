procstat-rs ([v0.1.6](https://github.com/kusumi/procstat-rs/releases/tag/v0.1.6))
========

## About

+ Ncurses based file monitor.

+ Rust version of [https://github.com/kusumi/procstat-go](https://github.com/kusumi/procstat-go).

## Requirements

Recent version of Rust

## Build

    $ make

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
    
    Commands:
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
      CTRL-l - Repaint whole screen
