#![no_std]
#![allow(non_camel_case_types)]
#![allow(clippy::uninit_assumed_init)]

pub use py32f0;

#[cfg(feature = "py32f002a")]
pub use py32f0::py32f002a as pac;
#[cfg(feature = "py32f003")]
pub use py32f0::py32f003 as pac;
#[cfg(feature = "py32f030")]
pub use py32f0::py32f030 as pac;

#[cfg(feature = "device-selected")]
pub mod adc;
#[cfg(feature = "device-selected")]
pub mod delay;
#[cfg(feature = "device-selected")]
pub mod gpio;
#[cfg(feature = "device-selected")]
pub mod i2c;
#[cfg(feature = "device-selected")]
pub mod prelude;
#[cfg(feature = "device-selected")]
pub mod pwm;
#[cfg(feature = "device-selected")]
pub mod rcc;
#[cfg(feature = "device-selected")]
pub mod serial;
#[cfg(feature = "device-selected")]
pub mod spi;
#[cfg(feature = "device-selected")]
pub mod time;
#[cfg(feature = "device-selected")]
pub mod timers;
#[cfg(feature = "device-selected")]
pub mod watchdog;
