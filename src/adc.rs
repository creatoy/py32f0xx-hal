//! # API for the Analog to Digital converter
//!
//! Currently implements oneshot conversion with variable sampling times.
//! Also references for the internal temperature sense, voltage
//! reference and battery sense are provided.
//!
//! ## Example
//! ``` no_run
//! use py32f0xx_hal as hal;
//!
//! use crate::hal::pac;
//! use crate::hal::prelude::*;
//! use crate::hal::adc::Adc;
//!
//! cortex_m::interrupt::free(|cs| {
//!     let mut p = pac::Peripherals::take().unwrap();
//!     let mut rcc = p.RCC.configure().freeze(&mut p.FLASH);
//!
//!     let gpioa = p.GPIOA.split(&mut rcc);
//!
//!     let mut led = gpioa.pa1.into_push_pull_pull_output(cs);
//!     let mut an_in = gpioa.pa0.into_analog(cs);
//!
//!     let mut delay = Delay::new(cp.SYST, &rcc);
//!
//!     let mut adc = Adc::new(p.ADC, &mut rcc);
//!
//!     loop {
//!         let val: u16 = adc.read(&mut an_in).unwrap();
//!         if val < ((1 << 8) - 1) {
//!             led.set_low();
//!         } else {
//!             led.set_high();
//!         }
//!         delay.delay_ms(50_u16);
//!     }
//! });
//! ```

// const VREFCAL: *const u16 = 0x1FFF_0F1A as *const u16;
const VREFINT_VAL: u16 = 1200;
const VTEMPCAL_LOW: *const u16 = 0x1FFF_0F14 as *const u16;
const VTEMPCAL_HIGH: *const u16 = 0x1FFF_0F18 as *const u16;
const VTEMPVAL_LOW: u16 = 30;
const VTEMPVAL_HIGH: u16 = 85;
const VTEMPVAL_DELTA: u16 = VTEMPVAL_HIGH - VTEMPVAL_LOW;

use core::ptr;

use embedded_hal::{
    adc::{Channel, OneShot},
    blocking::delay::DelayUs,
};

use crate::{
    delay::Delay,
    gpio::*,
    pac::{
        adc::{
            cfgr1::{ALIGN_A, RES_A},
            smpr::SMP_A,
            cfgr2::CKMODE_A,
        },
        ADC,
    },
    rcc::Rcc,
};

/// Analog to Digital converter interface
pub struct Adc {
    rb: ADC,
    sample_time: AdcSampleTime,
    align: AdcAlign,
    precision: AdcPrecision,
}

#[derive(Clone, Copy, Debug, PartialEq)]
/// ADC Clock mode, select adc clock source
///
pub enum AdcClockMode {
    /// PCLK
    Pclk,
    /// PCLK/2
    PclkDiv2,
    /// PCLK/4
    PclkDiv4,
    /// PCLK/8
    PclkDiv8,
    /// PCLK/16
    PclkDiv16,
    /// PCLK/32
    PclkDiv32,
    /// PCLK/64
    PclkDiv64,
    /// HSI
    Hsi,
    /// HSI/2
    HsiDiv2,
    /// HSI/4
    HsiDiv4,
    /// HSI/8
    HsiDiv8,
    /// HSI/16
    HsiDiv16,
    /// HSI/32
    HsiDiv32,
    /// HSI/64
    HsiDiv64,
}

impl AdcClockMode {
    /// Get the default clock mode (currently PCLK)
    pub fn default() -> Self {
        AdcClockMode::Pclk
    }
}

impl From<AdcClockMode> for CKMODE_A {
    fn from(value: AdcClockMode) -> Self {
        match value {
            AdcClockMode::Pclk => CKMODE_A::Pclk,
            AdcClockMode::PclkDiv2 => CKMODE_A::PclkDiv2,
            AdcClockMode::PclkDiv4 => CKMODE_A::PclkDiv4,
            AdcClockMode::PclkDiv8 => CKMODE_A::PclkDiv8,
            AdcClockMode::PclkDiv16 => CKMODE_A::PclkDiv16,
            AdcClockMode::PclkDiv32 => CKMODE_A::PclkDiv32,
            AdcClockMode::PclkDiv64 => CKMODE_A::PclkDiv64,
            AdcClockMode::Hsi => CKMODE_A::Hsi,
            AdcClockMode::HsiDiv2 => CKMODE_A::HsiDiv2,
            AdcClockMode::HsiDiv4 => CKMODE_A::HsiDiv4,
            AdcClockMode::HsiDiv8 => CKMODE_A::HsiDiv8,
            AdcClockMode::HsiDiv16 => CKMODE_A::HsiDiv16,
            AdcClockMode::HsiDiv32 => CKMODE_A::HsiDiv32,
            AdcClockMode::HsiDiv64 => CKMODE_A::HsiDiv64,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
/// ADC Sampling time
///
/// Options for the sampling time, each is T + 0.5 ADC clock cycles.
pub enum AdcSampleTime {
    /// 3.5 cycles sampling time
    T_3,
    /// 5.5 cycles sampling time
    T_5,
    /// 7.5 cycles sampling time
    T_7,
    /// 13.5 cycles sampling time
    T_13,
    /// 28.5 cycles sampling time
    T_28,
    /// 41.5 cycles sampling time
    T_41,
    /// 71.5 cycles sampling time
    T_71,
    /// 239.5 cycles sampling time
    T_239,
}

impl AdcSampleTime {
    /// Get the default sample time (currently 239.5 cycles)
    pub fn default() -> Self {
        AdcSampleTime::T_239
    }
}

impl From<AdcSampleTime> for SMP_A {
    fn from(val: AdcSampleTime) -> Self {
        match val {
            AdcSampleTime::T_3 => SMP_A::Cycles35,
            AdcSampleTime::T_5 => SMP_A::Cycles55,
            AdcSampleTime::T_7 => SMP_A::Cycles75,
            AdcSampleTime::T_13 => SMP_A::Cycles135,
            AdcSampleTime::T_28 => SMP_A::Cycles285,
            AdcSampleTime::T_41 => SMP_A::Cycles415,
            AdcSampleTime::T_71 => SMP_A::Cycles715,
            AdcSampleTime::T_239 => SMP_A::Cycles2395,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
/// ADC Result Alignment
pub enum AdcAlign {
    /// Left aligned results (most significant bits)
    ///
    /// Results in all precisions returning a value in the range 0-65535.
    /// Depending on the precision the result will step by larger or smaller
    /// amounts.
    Left,
    /// Right aligned results (least significant bits)
    ///
    /// Results in all precisions returning values from 0-(2^bits-1) in
    /// steps of 1.
    Right,
    /// Left aligned results without correction of 6bit values.
    ///
    /// Returns left aligned results exactly as shown in RM0091 Fig.37.
    /// Where the values are left aligned within the u16, with the exception
    /// of 6 bit mode where the value is left aligned within the first byte of
    /// the u16.
    LeftAsRM,
}

impl AdcAlign {
    /// Get the default alignment (currently right aligned)
    pub fn default() -> Self {
        AdcAlign::Right
    }
}

impl From<AdcAlign> for ALIGN_A {
    fn from(val: AdcAlign) -> Self {
        match val {
            AdcAlign::Left => ALIGN_A::Left,
            AdcAlign::Right => ALIGN_A::Right,
            AdcAlign::LeftAsRM => ALIGN_A::Left,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
/// ADC Sampling Precision
pub enum AdcPrecision {
    /// 12 bit precision
    B_12,
    /// 10 bit precision
    B_10,
    /// 8 bit precision
    B_8,
    /// 6 bit precision
    B_6,
}

impl AdcPrecision {
    /// Get the default precision (currently 12 bit precision)
    pub fn default() -> Self {
        AdcPrecision::B_12
    }
}

impl From<AdcPrecision> for RES_A {
    fn from(val: AdcPrecision) -> Self {
        match val {
            AdcPrecision::B_12 => RES_A::TwelveBit,
            AdcPrecision::B_10 => RES_A::TenBit,
            AdcPrecision::B_8 => RES_A::EightBit,
            AdcPrecision::B_6 => RES_A::SixBit,
        }
    }
}

macro_rules! adc_pins {
    ($($pin:ty => $chan:expr),+ $(,)*) => {
        $(
            impl Channel<Adc> for $pin {
                type ID = u8;

                fn channel() -> u8 { $chan }
            }
        )+
    };
}

adc_pins!(
    gpioa::PA0<Analog> => 0_u8,
    gpioa::PA1<Analog> => 1_u8,
    gpioa::PA2<Analog> => 2_u8,
    gpioa::PA3<Analog> => 3_u8,
    gpioa::PA4<Analog> => 4_u8,
    gpioa::PA5<Analog> => 5_u8,
    gpioa::PA6<Analog> => 6_u8,
    gpioa::PA7<Analog> => 7_u8,
    gpiob::PB0<Analog> => 8_u8,
    gpiob::PB1<Analog> => 9_u8,
);

#[derive(Debug, Default)]
/// Internal temperature sensor (ADC Channel 11)
pub struct VTemp;

#[derive(Debug, Default)]
/// Internal voltage reference (ADC Channel 12)
pub struct VRef;

adc_pins!(
    VTemp => 11_u8,
    VRef  => 12_u8,
);

impl VTemp {
    /// Init a new VTemp
    pub fn new() -> Self {
        VTemp::default()
    }

    /// Enable the internal temperature sense, this has a wake up time
    /// t<sub>START</sub> which can be found in your micro's datasheet, you
    /// must wait at least that long after enabling before taking a reading.
    /// Remember to disable when not in use.
    pub fn enable(&mut self, adc: &mut Adc) {
        adc.rb.ccr.modify(|_, w| w.tsen().set_bit());
    }

    /// Disable the internal temperature sense.
    pub fn disable(&mut self, adc: &mut Adc) {
        adc.rb.ccr.modify(|_, w| w.tsen().clear_bit());
    }

    /// Checks if the temperature sensor is enabled, does not account for the
    /// t<sub>START</sub> time however.
    pub fn is_enabled(&self, adc: &Adc) -> bool {
        adc.rb.ccr.read().tsen().bit_is_set()
    }

    fn convert_temp(vtemp: u16) -> i16 {
        let vtempcal_low = unsafe { ptr::read(VTEMPCAL_LOW) } as i32;
        let vtempcal_high = unsafe { ptr::read(VTEMPCAL_HIGH) } as i32;
        ((vtemp as i32 - vtempcal_low) * 100 * (VTEMPVAL_DELTA as i32) / (vtempcal_high - vtempcal_low)
            + 3000) as i16
    }

    /// Read the value of the internal temperature sensor and return the
    /// result in 10ths of a degree centigrade.
    ///
    /// Given a delay reference it will attempt to restrict to the
    /// minimum delay needed to ensure a 10 us t<sub>START</sub> value.
    /// Otherwise it will approximate the required delay using ADC reads.
    pub fn read(adc: &mut Adc, delay: Option<&mut Delay>) -> i16 {
        let mut vtemp = Self::new();
        let vtemp_preenable = vtemp.is_enabled(adc);

        if !vtemp_preenable {
            vtemp.enable(adc);

            if let Some(dref) = delay {
                dref.delay_us(2_u16);
            } else {
                // Double read of vdda to allow sufficient startup time for the temp sensor
                VRef::read_vdda(adc);
            }
        }
        // let vdda = VRef::read_vdda(adc);

        let prev_cfg = adc.default_cfg();

        let vtemp_val = adc.read(&mut vtemp).unwrap();

        if !vtemp_preenable {
            vtemp.disable(adc);
        }

        adc.restore_cfg(prev_cfg);

        Self::convert_temp(vtemp_val)
    }
}

impl VRef {
    /// Init a new VRef
    pub fn new() -> Self {
        VRef::default()
    }

    /// Enable the internal voltage reference, remember to disable when not in use.
    pub fn enable(&mut self, adc: &mut Adc) {
        adc.rb.ccr.modify(|_, w| w.vrefen().set_bit());
    }

    /// Disable the internal reference voltage.
    pub fn disable(&mut self, adc: &mut Adc) {
        adc.rb.ccr.modify(|_, w| w.vrefen().clear_bit());
    }

    /// Returns if the internal voltage reference is enabled.
    pub fn is_enabled(&self, adc: &Adc) -> bool {
        adc.rb.ccr.read().vrefen().bit_is_set()
    }

    /// Reads the value of VDDA in milli-volts
    pub fn read_vdda(adc: &mut Adc) -> u16 {
        let mut vref = Self::new();

        let prev_cfg = adc.default_cfg();

        let vref_val: u32 = if vref.is_enabled(adc) {
            adc.read(&mut vref).unwrap()
        } else {
            vref.enable(adc);

            let ret = adc.read(&mut vref).unwrap();

            vref.disable(adc);
            ret
        };

        adc.restore_cfg(prev_cfg);
        
        ((u32::from(VREFINT_VAL) * 4095) / vref_val) as u16
    }
}

/// A stored ADC config, can be restored by using the `Adc::restore_cfg` method
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct StoredConfig(AdcSampleTime, AdcAlign, AdcPrecision);

impl Adc {
    /// Init a new Adc
    ///
    /// Sets all configurable parameters to defaults, enables the HSI14 clock
    /// for the ADC if it is not already enabled and performs a boot time
    /// calibration. As such this method may take an appreciable time to run.
    pub fn new(adc: ADC, rcc: &mut Rcc, ckmode: AdcClockMode) -> Self {
        let mut s = Self {
            rb: adc,
            sample_time: AdcSampleTime::default(),
            align: AdcAlign::default(),
            precision: AdcPrecision::default(),
        };
        s.select_clock(rcc, ckmode);
        s.calibrate();
        s
    }

    /// Saves a copy of the current ADC config
    pub fn save_cfg(&mut self) -> StoredConfig {
        StoredConfig(self.sample_time, self.align, self.precision)
    }

    /// Restores a stored config
    pub fn restore_cfg(&mut self, cfg: StoredConfig) {
        self.sample_time = cfg.0;
        self.align = cfg.1;
        self.precision = cfg.2;
    }

    /// Resets the ADC config to default, returning the existing config as
    /// a stored config.
    pub fn default_cfg(&mut self) -> StoredConfig {
        let cfg = self.save_cfg();
        self.sample_time = AdcSampleTime::default();
        self.align = AdcAlign::default();
        self.precision = AdcPrecision::default();
        cfg
    }

    /// Set the Adc sampling time
    ///
    /// Options can be found in [AdcSampleTime](crate::adc::AdcSampleTime).
    pub fn set_sample_time(&mut self, t_samp: AdcSampleTime) {
        self.sample_time = t_samp;
    }

    /// Set the Adc result alignment
    ///
    /// Options can be found in [AdcAlign](crate::adc::AdcAlign).
    pub fn set_align(&mut self, align: AdcAlign) {
        self.align = align;
    }

    /// Set the Adc precision
    ///
    /// Options can be found in [AdcPrecision](crate::adc::AdcPrecision).
    pub fn set_precision(&mut self, precision: AdcPrecision) {
        self.precision = precision;
    }

    /// Returns the largest possible sample value for the current settings
    pub fn max_sample(&self) -> u16 {
        match self.align {
            AdcAlign::Left => u16::max_value(),
            AdcAlign::LeftAsRM => match self.precision {
                AdcPrecision::B_6 => u16::from(u8::max_value()),
                _ => u16::max_value(),
            },
            AdcAlign::Right => match self.precision {
                AdcPrecision::B_12 => (1 << 12) - 1,
                AdcPrecision::B_10 => (1 << 10) - 1,
                AdcPrecision::B_8 => (1 << 8) - 1,
                AdcPrecision::B_6 => (1 << 6) - 1,
            },
        }
    }

    /// Read the value of a channel and converts the result to milli-volts
    pub fn read_abs_mv<PIN: Channel<Adc, ID = u8>>(&mut self, pin: &mut PIN) -> u16 {
        let vdda = u32::from(VRef::read_vdda(self));
        let val: u32 = self.read(pin).unwrap();
        let max_samp = u32::from(self.max_sample());

        (val * vdda / max_samp) as u16
    }

    fn calibrate(&mut self) {
        /* Ensure that ADEN = 0 */
        if self.rb.cr.read().aden().is_enabled() {
            /* Clear ADEN by stop conversion */
            self.rb.cr.modify(|_, w| w.adstp().stop_conversion());
            while self.rb.cr.read().adstp().is_stopping() {}
        }
        while self.rb.cr.read().aden().is_enabled() {}

        let dmacfg = self.rb.cfgr1.read().dmacfg().bit();
        let dmaen = self.rb.cfgr1.read().dmaen().is_enabled();

        /* Clear DMAEN */
        // self.rb.cfgr1.modify(|_, w| w.dmaen().disabled());

        /* Start calibration by setting ADCAL */
        self.rb.cr.modify(|_, w| w.adcal().start_calibration());

        /* Wait until calibration is finished and ADCAL = 0 */
        while self.rb.cr.read().adcal().is_calibrating() {}

        self.rb.cfgr1.modify(|_, w| w.dmacfg().bit(dmacfg).dmaen().bit(dmaen));
    }

    fn select_clock(&mut self, rcc: &mut Rcc, ckmode: AdcClockMode) {
        rcc.regs.apbrstr2.modify(|_, w| w.adcrst().reset());
        rcc.regs.apbrstr2.modify(|_, w| w.adcrst().clear_bit());
        rcc.regs.apbenr2.modify(|_, w| w.adcen().enabled());
        self.rb.cfgr2.modify(|_, w| w.ckmode().variant(ckmode.into()));
    }

    /// Apply config settings
    fn apply_cfg(&mut self) {
        self.rb
            .smpr
            .write(|w| w.smp().variant(self.sample_time.into()));
        self.rb.cfgr1.modify(|_, w| {
            w.res()
                .variant(self.precision.into())
                .align()
                .variant(self.align.into())
                .wait().disabled()
        });

    }

    fn power_up(&mut self, chan: u8) {
        self.apply_cfg();
        self.rb.chselr.write(|w| unsafe { w.bits(1_u32 << chan) });
        self.rb.cr.modify(|_, w| w.aden().enabled());
    }

    fn power_down(&mut self) {
        if self.rb.cr.read().aden().is_enabled() {
            self.rb.cr.modify(|_, w| w.adstp().stop_conversion());
            while self.rb.cr.read().adstp().is_stopping() {}
        }
    }

    fn convert(&mut self) -> u16 {
        self.rb.cr.modify(|_, w| w.adstart().start_conversion());
        while self.rb.isr.read().eoc().is_not_complete() {}

        let res = self.rb.dr.read().bits() as u16;
        if self.align == AdcAlign::Left && self.precision == AdcPrecision::B_6 {
            res << 8
        } else {
            res
        }
    }
}

impl<WORD, PIN> OneShot<Adc, WORD, PIN> for Adc
where
    WORD: From<u16>,
    PIN: Channel<Adc, ID = u8>,
{
    type Error = ();

    fn read(&mut self, _pin: &mut PIN) -> nb::Result<WORD, Self::Error> {
        self.power_up(PIN::channel());
        let res = self.convert();
        self.power_down();
        Ok(res.into())
    }
}
