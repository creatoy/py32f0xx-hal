use core::ops::Deref;

use embedded_hal::blocking::i2c::{Read, Write, WriteRead};

use crate::{
    gpio::*,
    rcc::Rcc,
    time::{Hertz, KiloHertz, U32Ext},
};

/// I2C abstraction
pub struct I2c<I2C, SCLPIN, SDAPIN> {
    i2c: I2C,
    pins: (SCLPIN, SDAPIN),
}

pub trait SclPin<I2C> {}
pub trait SdaPin<I2C> {}

macro_rules! i2c_pins {
    ($($I2C:ident => {
        scl => [$($scl:ty),+ $(,)*],
        sda => [$($sda:ty),+ $(,)*],
    })+) => {
        $(
            $(
                impl SclPin<crate::pac::$I2C> for $scl {}
            )+
            $(
                impl SdaPin<crate::pac::$I2C> for $sda {}
            )+
        )+
    }
}

// TODO: Double check if all parts aexcept f002b support these
#[cfg(any(feature = "py32f030", feature = "py32f003", feature = "py32f002a"))]
i2c_pins! {
    I2C => {
        scl => [
            gpioa::PA3<Alternate<AF12>>,
            gpioa::PA8<Alternate<AF12>>,
            gpioa::PA9<Alternate<AF6>>,
            gpioa::PA10<Alternate<AF12>>,
            gpioa::PA11<Alternate<AF6>>,
            gpiob::PB6<Alternate<AF6>>,
            gpiob::PB8<Alternate<AF6>>,
            gpiof::PF1<Alternate<AF12>>
        ],
        sda => [
            gpioa::PA2<Alternate<AF12>>,
            gpioa::PA7<Alternate<AF12>>,
            gpioa::PA9<Alternate<AF12>>,
            gpioa::PA10<Alternate<AF6>>,
            gpioa::PA12<Alternate<AF6>>,
            gpiob::PB7<Alternate<AF6>>,
            gpiob::PB8<Alternate<AF12>>,
            gpiof::PF0<Alternate<AF12>>
        ],
    }
}

#[cfg(feature = "py32f002b")]
i2c_pins! {
    I2C => {
        scl => [
            gpioa::PA2<Alternate<AF6>>,
            gpiob::PB3<Alternate<AF6>>,
        ],
        sda => [
            gpiob::PB4<Alternate<AF6>>,
            gpiob::PB6<Alternate<AF6>>,
        ],
    }
}

#[derive(Debug)]
pub enum Error {
    OVERRUN,
    NACK,
    BUS,
    PEC,
}

macro_rules! i2c {
    ($($I2C:ident: ($i2c:ident, $i2cXen:ident, $i2cXrst:ident, $apbenr:ident, $apbrstr:ident),)+) => {
        $(
            use crate::pac::$I2C;
            impl<SCLPIN, SDAPIN> I2c<$I2C, SCLPIN, SDAPIN> {
                pub fn $i2c(i2c: $I2C, pins: (SCLPIN, SDAPIN), speed: KiloHertz, rcc: &mut Rcc) -> Self
                where
                    SCLPIN: SclPin<$I2C>,
                    SDAPIN: SdaPin<$I2C>,
                {
                    // Enable clock for I2C
                    rcc.regs.$apbenr.modify(|_, w| w.$i2cXen().set_bit());

                    // Reset I2C
                    rcc.regs.$apbrstr.modify(|_, w| w.$i2cXrst().set_bit());
                    rcc.regs.$apbrstr.modify(|_, w| w.$i2cXrst().clear_bit());
                    I2c { i2c, pins }.i2c_init(rcc.clocks.pclk(), speed)
                }
            }
        )+
    }
}

i2c! {
    I2C: (i2c, i2cen, i2crst, apbenr1, apbrstr1),
}

// It's s needed for the impls, but rustc doesn't recognize that
#[allow(dead_code)]
type I2cRegisterBlock = crate::pac::i2c::RegisterBlock;

impl<I2C, SCLPIN, SDAPIN> I2c<I2C, SCLPIN, SDAPIN>
where
    I2C: Deref<Target = I2cRegisterBlock>,
{
    fn i2c_init(self, freq: Hertz, speed: KiloHertz) -> Self {
        // Make sure the I2C unit is disabled so we can configure it
        self.i2c.cr1.modify(|_, w| w.pe().clear_bit());

        let f = freq.0 / 1_000_000;

        self.i2c
            .cr2
            .write(|w| unsafe { w.freq().bits(f.clamp(4, 48) as u8) });

        // Normal I2C speeds use a different scaling than fast mode below
        let (f_s, ccr) = if speed <= 100_u32.khz() {
            // This is a normal I2C mode
            (false, freq.0 / (speed.0 * 2))
        } else {
            // This is a fast I2C mode
            (
                true,
                if self.i2c.ccr.read().duty().bit_is_set() {
                    freq.0 / (speed.0 * 25)
                } else {
                    freq.0 / (speed.0 * 3)
                },
            )
        };
        self.i2c
            .ccr
            .modify(|_, w| unsafe { w.f_s().bit(f_s).ccr().bits(ccr.clamp(4, 4095) as u16) });

        // Enable the I2C processing
        self.i2c.cr1.modify(|_, w| w.pe().set_bit());

        self
    }

    pub fn release(self) -> (I2C, (SCLPIN, SDAPIN)) {
        (self.i2c, self.pins)
    }

    fn check_and_clear_error_flags(&self, sr: &crate::pac::i2c::sr1::R) -> Result<(), Error> {
        // If we have a set pec error flag, clear it and return an PEC error
        if sr.pecerr().bit_is_set() {
            self.i2c.sr1.write(|w| w.pecerr().clear_bit());
            return Err(Error::PEC);
        }

        // If we have a set overrun flag, clear it and return an OVERRUN error
        if sr.ovr().bit_is_set() {
            self.i2c.sr1.write(|w| w.ovr().clear_bit());
            return Err(Error::OVERRUN);
        }

        // If we have a set arbitration error or bus error flag, clear it and return an BUS error
        if sr.arlo().bit_is_set() | sr.berr().bit_is_set() {
            self.i2c
                .sr1
                .write(|w| w.arlo().clear_bit().berr().clear_bit());
            return Err(Error::BUS);
        }

        // If we received a NACK, then signal as a NACK error
        if sr.af().bit_is_set() {
            self.i2c.sr1.write(|w| w.af().clear_bit());
            return Err(Error::NACK);
        }

        Ok(())
    }

    fn send_byte(&self, byte: u8) -> Result<(), Error> {
        // Wait until we're ready for sending
        loop {
            let sr = self.i2c.sr1.read();
            self.check_and_clear_error_flags(&sr)?;
            if sr.txe().bit_is_set() {
                break;
            }
        }

        // Push out a byte of data
        self.i2c.dr.write(|w| unsafe { w.bits(u32::from(byte)) });

        self.check_and_clear_error_flags(&self.i2c.sr1.read())?;
        Ok(())
    }

    fn recv_byte(&self) -> Result<u8, Error> {
        loop {
            let sr = self.i2c.sr1.read();
            self.check_and_clear_error_flags(&sr)?;
            if sr.rxne().bit_is_set() {
                break;
            }
        }

        let value = self.i2c.dr.read().bits() as u8;
        Ok(value)
    }
}

impl<I2C, SCLPIN, SDAPIN> WriteRead for I2c<I2C, SCLPIN, SDAPIN>
where
    I2C: Deref<Target = I2cRegisterBlock>,
{
    type Error = Error;

    fn write_read(&mut self, addr: u8, bytes: &[u8], buffer: &mut [u8]) -> Result<(), Error> {
        // Set up current slave address for writing and disable autoending
        self.i2c.oar1.modify(|_, w| w.add().bits(addr));

        // Send a START condition
        self.i2c.cr1.modify(|_, w| w.start().set_bit());

        // Wait until the transmit buffer is empty and there hasn't been any error condition
        loop {
            let sr = self.i2c.sr1.read();
            self.check_and_clear_error_flags(&sr)?;
            if sr.txe().bit_is_set() {
                break;
            }
        }

        // Send out all individual bytes
        for c in bytes {
            self.send_byte(*c)?;
        }

        // Wait until data was sent
        loop {
            let sr = self.i2c.sr1.read();
            self.check_and_clear_error_flags(&sr)?;
            if sr.btf().bit_is_set() {
                break;
            }
        }

        // Set up current address for reading
        self.i2c.oar1.modify(|_, w| w.add().bits(addr));

        // Send another START condition
        self.i2c.cr1.modify(|_, w| w.start().set_bit());

        // Send the autoend after setting the start to get a restart
        // self.i2c.cr2.modify(|_, w| w.autoend().set_bit());

        // Now read in all bytes
        for c in buffer.iter_mut() {
            *c = self.recv_byte()?;
        }

        // Check and clear flags if they somehow ended up set
        self.check_and_clear_error_flags(&self.i2c.sr1.read())?;

        Ok(())
    }
}

impl<I2C, SCLPIN, SDAPIN> Read for I2c<I2C, SCLPIN, SDAPIN>
where
    I2C: Deref<Target = I2cRegisterBlock>,
{
    type Error = Error;

    fn read(&mut self, addr: u8, buffer: &mut [u8]) -> Result<(), Error> {
        // Set up current address for reading
        self.i2c.oar1.modify(|_, w| w.add().bits(addr));

        // Send a START condition
        self.i2c.cr1.modify(|_, w| w.start().set_bit());

        // Send the autoend after setting the start to get a restart
        // self.i2c.cr2.modify(|_, w| w.autoend().set_bit());

        // Now read in all bytes
        for c in buffer.iter_mut() {
            *c = self.recv_byte()?;
        }

        // Check and clear flags if they somehow ended up set
        self.check_and_clear_error_flags(&self.i2c.sr1.read())?;

        Ok(())
    }
}

impl<I2C, SCLPIN, SDAPIN> Write for I2c<I2C, SCLPIN, SDAPIN>
where
    I2C: Deref<Target = I2cRegisterBlock>,
{
    type Error = Error;

    fn write(&mut self, addr: u8, bytes: &[u8]) -> Result<(), Error> {
        // Set up current slave address for writing and enable autoending
        self.i2c.oar1.modify(|_, w| w.add().bits(addr));

        // Send a START condition
        self.i2c.cr1.modify(|_, w| w.start().set_bit());

        // Send out all individual bytes
        for c in bytes {
            self.send_byte(*c)?;
        }

        // Check and clear flags if they somehow ended up set
        self.check_and_clear_error_flags(&self.i2c.sr1.read())?;

        Ok(())
    }
}
