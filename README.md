py32f0xx-hal
=============

> This repo is modified from [stm32f0xx-hal](https://github.com/stm32-rs/stm32f0xx-hal)  

> **NOTE: The function is not fully tested, and you are responsible for any problems with the use of this repository.**

Known issue:
    - I2C not work

[![Continuous integration](https://github.com/creatoy/py32f0xx-hal/workflows/Continuous%20integration/badge.svg)](https://github.com/creatoy/py32f0xx-hal)
[![Crates.io](https://img.shields.io/crates/v/py32f0xx-hal.svg)](https://crates.io/crates/py32f0xx-hal)
[![docs.rs](https://docs.rs/py32f0xx-hal/badge.svg)](https://docs.rs/py32f0xx-hal/)

[_py32f0xx-hal_](https://github.com/creatoy/py32f0xx-hal) contains a hardware abstraction on top of the peripheral access API for the puyasemi PY32F0xx family of microcontrollers.

Collaboration on this crate is highly welcome, as are pull requests!

Supported
------------------------

* __py32f030__ (py32f030xx4, py32f030xx6, py32f030xx7, py32f030xx8)  
* __py32f003__ (py32f003xx4, py32f003xx6, py32f030xx8)  
* __py32f002a__ (py32f002ax5)  
* __py32f002b__ (py32f002bx5)  

Getting Started
---------------
The `examples` folder contains several example programs. To compile them, one must specify the target device as cargo feature:
```
$ cargo build --features=py32f002ax5 --example=blinky
```

To use py32f0xx-hal as a dependency in a standalone project the target device feature must be specified in the `Cargo.toml` file:
```
[dependencies]
cortex-m = "0.7.7"
cortex-m-rt = "0.7.3"
py32f0xx-hal = { version = "0.1.0", features = ["py32f002ax5"]}
```

If you are unfamiliar with embedded development using Rust, there are a number of fantastic resources available to help.

- [Embedded Rust Documentation](https://docs.rust-embedded.org/)  
- [The Embedded Rust Book](https://docs.rust-embedded.org/book/)  
- [Rust Embedded FAQ](https://docs.rust-embedded.org/faq.html)  
- [rust-embedded/awesome-embedded-rust](https://github.com/rust-embedded/awesome-embedded-rust)


Minimum supported Rust version
------------------------------

The minimum supported Rust version is the latest stable release. Older versions may compile, especially when some features are not used in your application.

Changelog
---------

See [CHANGELOG.md](CHANGELOG.md).


License
-------

0-Clause BSD License, see [LICENSE-0BSD.txt](LICENSE-0BSD.txt) for more details.
