pub use embedded_hal::prelude::*;

// TODO for some reason, watchdog isn't in the embedded_hal prelude
pub use embedded_hal::watchdog::Watchdog as _py32f0xx_hal_embedded_hal_watchdog_Watchdog;
pub use embedded_hal::watchdog::WatchdogEnable as _py32f0xx_hal_embedded_hal_watchdog_WatchdogEnable;

pub use embedded_hal::adc::OneShot as _embedded_hal_adc_OneShot;

pub use embedded_hal::digital::v2::InputPin as _embedded_hal_gpio_InputPin;
pub use embedded_hal::digital::v2::OutputPin as _embedded_hal_gpio_OutputPin;
pub use embedded_hal::digital::v2::StatefulOutputPin as _embedded_hal_gpio_StatefulOutputPin;
pub use embedded_hal::digital::v2::ToggleableOutputPin as _embedded_hal_gpio_ToggleableOutputPin;

pub use crate::gpio::GpioExt as _py32f0xx_hal_gpio_GpioExt;
pub use crate::rcc::RccExt as _py32f0xx_hal_rcc_RccExt;
pub use crate::time::U32Ext as _py32f0xx_hal_time_U32Ext;
