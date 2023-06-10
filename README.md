py32f0xx-hal
=============

> This repo is modified from [stm32f0xx-hal](https://github.com/stm32-rs/stm32f0xx-hal)  

> Currently only support the PY32f030 (the py32f002axx py32f003xx and py32f030xx peripherals are same, so you can controll all these chips using this hal)

> **NOTE: The function is not fully tested, and you are responsible for any problems with the use of this repository.**

Known issue:
    - I2C not work

[![Continuous integration](https://github.com/creatoy/py32f0xx-hal/workflows/Continuous%20integration/badge.svg)](https://github.com/creatoy/py32f0xx-hal)
[![Crates.io](https://img.shields.io/crates/v/py32f0xx-hal.svg)](https://crates.io/crates/py32f0xx-hal)
[![docs.rs](https://docs.rs/py32f0xx-hal/badge.svg)](https://docs.rs/py32f0xx-hal/)

[_py32f0xx-hal_](https://github.com/creatoy/py32f0xx-hal) contains a hardware abstraction on top of the peripheral access API for the puyasemi PY32F0xx family of microcontrollers.

Collaboration on this crate is highly welcome, as are pull requests!

Supported Configurations
------------------------

* __py32f030__ (py32f030x4, py32f030x6, py32f030x8, py32f030xc)  

> py32f002axx and py32f003xx are supported due to the peripherals are same


Getting Started
---------------
The `examples` folder contains several example programs. To compile them, one must specify the target device as cargo feature:
```
$ cargo build --features=py32f030k16t
```

To use py32f0xx-hal as a dependency in a standalone project the target device feature must be specified in the `Cargo.toml` file:
```
[dependencies]
cortex-m = "0.7.7"
cortex-m-rt = "0.7.3"
py32f0xx-hal = { version = "0.0.1", features = ["py32f030k16t"]}
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
