# wcolor
[![Latest Version](https://img.shields.io/crates/v/wcolor.svg)](https://crates.io/crates/wcolor)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Lightweight color picker for Windows written in rust.  
Get color from anywhere on your screen with mouse click. 

## Installation
``` shell
cargo install wcolor
```

# Usage

``` shell
wcolor.exe [FLAGS] [OPTIONS]

FLAGS:
    -c, --clipboard
    -h, --help          Prints help information
    -n, --no-preview
    -V, --version       Prints version information

OPTIONS:
    -f, --format <format>     [default: HEX]  [possible values: HEX, hex, RGB]
```

Inspired by [xcolor](https://github.com/Soft/xcolor)
