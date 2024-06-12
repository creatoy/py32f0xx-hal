//! API for the integrated USART ports
//!
//! This only implements the usual asynchronous bidirectional 8-bit transfers.
//!
//! It's possible to use a read-only/write-only serial implementation with
//! `usartXrx`/`usartXtx`.
//!
//! # Examples
//! Echo
//! ``` no_run
//! use py32f0xx_hal as hal;
//!
//! use crate::hal::prelude::*;
//! use crate::hal::serial::Serial;
//! use crate::hal::pac;
//!
//! use nb::block;
//!
//! cortex_m::interrupt::free(|cs| {
//!     let rcc = p.RCC.configure().sysclk(48.mhz()).freeze();
//!
//!     let gpioa = p.GPIOA.split(&mut rcc);
//!
//!     let tx = gpioa.pa9.into_alternate_af1(cs);
//!     let rx = gpioa.pa10.into_alternate_af1(cs);
//!
//!     let mut serial = Serial::usart1(p.USART1, (tx, rx), 115_200.bps(), &mut rcc);
//!
//!     loop {
//!         let received = block!(serial.read()).unwrap();
//!         block!(serial.write(received)).ok();
//!     }
//! });
//! ```
//!
//! Hello World
//! ``` no_run
//! use py32f0xx_hal as hal;
//!
//! use crate::hal::prelude::*;
//! use crate::hal::serial::Serial;
//! use crate::hal::pac;
//!
//! use nb::block;
//!
//! cortex_m::interrupt::free(|cs| {
//!     let rcc = p.RCC.configure().sysclk(48.mhz()).freeze();
//!
//!     let gpioa = p.GPIOA.split(&mut rcc);
//!
//!     let tx = gpioa.pa9.into_alternate_af1(cs);
//!
//!     let mut serial = Serial::usart1tx(p.USART1, tx, 115_200.bps(), &mut rcc);
//!
//!     loop {
//!         serial.write_str("Hello World!\r\n");
//!     }
//! });
//! ```

use core::{
    convert::Infallible,
    fmt::{Result, Write},
    ops::Deref,
};

use embedded_hal::prelude::*;

use crate::{rcc::Rcc, time::Bps};
use crate::gpio::{gpioa::*, gpiob::*, gpiof::*, Alternate, AF0, AF1, AF8};

#[cfg(any(feature = "py32f030", feature = "py32f003"))]
use crate::gpio::{AF3, AF4, AF8, AF9};

use core::marker::PhantomData;

/// Serial error
#[non_exhaustive]
#[derive(Debug)]
pub enum Error {
    /// Framing error
    Framing,
    /// Noise error
    Noise,
    /// RX buffer overrun
    Overrun,
    /// Parity check error
    Parity,
}

/// Interrupt event
pub enum Event {
    /// New data has been received
    Rxne,
    /// New data can be sent
    Txe,
    /// Idle line state detected
    Idle,
}

pub trait TxPin<USART> {}
pub trait RxPin<USART> {}

/// Macro to implement `TxPin` / `RxPin` for a certain pin, using a certain
/// alternative function and for a certain serial peripheral.
macro_rules! impl_pins {
    ($($pin:ident, $af:ident, $instance:ident, $trait:ident;)*) => {
        $(
            impl $trait<crate::pac::$instance> for $pin<Alternate<$af>> {}
        )*
    }
}

#[cfg(any(
    feature = "lqfp32k1",
    feature = "lqfp32k2",
    feature = "qfn32k2",
))]
impl_pins!(
    PA8, AF8, USART1, RxPin;
    PA8, AF9, USART2, RxPin;
    PA9, AF1, USART1, TxPin;
    PA9, AF4, USART2, TxPin;
    PA9, AF8, USART1, RxPin;
    PA10, AF1, USART1, RxPin;
    PA10, AF4, USART2, RxPin;
    PA10, AF8, USART1, TxPin;

    PB6, AF0, USART1, TxPin;
    PB6, AF4, USART2, TxPin;
    PB7, AF0, USART1, RxPin;
    PB7, AF4, USART2, RxPin;

    PF3, AF0, USART1, TxPin;
    PF3, AF4, USART2, TxPin;
);

#[cfg(any(
    feature = "lqfp32k1",
    feature = "lqfp32k2",
    feature = "qfn32k2",

    feature = "ssop24e1",
    feature = "ssop24e2",
))]
impl_pins!(
    PA0, AF9, USART2, TxPin;
    PA1, AF9, USART2, RxPin;
    PA2, AF1, USART1, TxPin;
    PA2, AF4, USART2, TxPin;
    PA3, AF1, USART1, RxPin;
    PA3, AF4, USART2, RxPin;
    PA4, AF9, USART2, TxPin;
    PA5, AF9, USART2, RxPin;
    PA7, AF8, USART1, TxPin;
    PA7, AF9, USART2, TxPin;
    PA13, AF8, USART1, RxPin;
    PA14, AF1, USART1, TxPin;
    PA14, AF4, USART2, TxPin;
    PA15, AF1, USART1, RxPin;
    PA15, AF4, USART2, RxPin;
    PB2, AF0, USART1, RxPin;
    PB2, AF3, USART2, RxPin;
    PB8, AF4, USART2, TxPin;
    PB8, AF8, USART1, TxPin;
    PF0, AF4, USART2, RxPin;
    PF0, AF8, USART1, RxPin;
    PF0, AF9, USART2, TxPin;
    PF1, AF4, USART2, TxPin;
    PF1, AF8, USART1, TxPin;
    PF1, AF9, USART2, RxPin;
    PF2, AF4, USART2, RxPin;
);

#[cfg(feature = "py32f002a")]
impl_pins!(
    PA2, AF1, USART1, TxPin;
    PA3, AF1, USART1, RxPin;

    PA7, AF8, USART1, TxPin;
    PA8, AF8, USART1, RxPin;

    PA9, AF1, USART1, TxPin;
    PA9, AF8, USART1, RxPin;

    PA10, AF1, USART1, RxPin;
    PA10, AF8, USART1, TxPin;

    PA13, AF8, USART1, RxPin;
    PA14, AF1, USART1, TxPin;
    
    PB2, AF0, USART1, RxPin;
    PB6, AF0, USART1, TxPin;

    PF0, AF8, USART1, RxPin;
    PF1, AF8, USART1, TxPin;
);

/// Serial abstraction
pub struct Serial<USART, TXPIN, RXPIN> {
    usart: USART,
    pins: (TXPIN, RXPIN),
}

// Common register
type SerialRegisterBlock = crate::pac::usart1::RegisterBlock;

/// Serial receiver
pub struct Rx<USART> {
    usart: *const SerialRegisterBlock,
    _instance: PhantomData<USART>,
}

// NOTE(unsafe) Required to allow protected shared access in handlers
unsafe impl<USART> Send for Rx<USART> {}

/// Serial transmitter
pub struct Tx<USART> {
    usart: *const SerialRegisterBlock,
    _instance: PhantomData<USART>,
}

// NOTE(unsafe) Required to allow protected shared access in handlers
unsafe impl<USART> Send for Tx<USART> {}

macro_rules! usart {
    ($($USART:ident: ($usart:ident, $usarttx:ident, $usartrx:ident, $usartXen:ident, $apbenr:ident),)+) => {
        $(
            use crate::pac::$USART;
            impl<TXPIN, RXPIN> Serial<$USART, TXPIN, RXPIN>
            where
                TXPIN: TxPin<$USART>,
                RXPIN: RxPin<$USART>,
            {
                /// Creates a new serial instance
                pub fn $usart(usart: $USART, pins: (TXPIN, RXPIN), baud_rate: Bps, rcc: &mut Rcc) -> Self
                {
                    let mut serial = Serial { usart, pins };
                    serial.configure(baud_rate, rcc);
                    // Enable transmission and receiving
                    serial.usart.cr1.modify(|_, w| w.te().set_bit().re().set_bit().ue().set_bit());
                    serial
                }
            }

            impl<TXPIN> Serial<$USART, TXPIN, ()>
            where
                TXPIN: TxPin<$USART>,
            {
                /// Creates a new tx-only serial instance
                pub fn $usarttx(usart: $USART, txpin: TXPIN, baud_rate: Bps, rcc: &mut Rcc) -> Self
                {
                    let rxpin = ();
                    let mut serial = Serial { usart, pins: (txpin, rxpin) };
                    serial.configure(baud_rate, rcc);
                    // Enable transmission
                    serial.usart.cr1.modify(|_, w| w.te().set_bit().ue().set_bit());
                    serial
                }
            }

            impl<RXPIN> Serial<$USART, (), RXPIN>
            where
                RXPIN: RxPin<$USART>,
            {
                /// Creates a new rx-only serial instance
                pub fn $usartrx(usart: $USART, rxpin: RXPIN, baud_rate: Bps, rcc: &mut Rcc) -> Self
                {
                    let txpin = ();
                    let mut serial = Serial { usart, pins: (txpin, rxpin) };
                    serial.configure(baud_rate, rcc);
                    // Enable receiving
                    serial.usart.cr1.modify(|_, w| w.re().set_bit().ue().set_bit());
                    serial
                }
            }

            impl<TXPIN, RXPIN> Serial<$USART, TXPIN, RXPIN> {
                fn configure(&mut self, baud_rate: Bps, rcc: &mut Rcc) {
                    // Enable clock for USART
                    rcc.regs.$apbenr.modify(|_, w| w.$usartXen().set_bit());

                    // Calculate correct baudrate divisor on the fly
                    let brr = rcc.clocks.pclk().0 / baud_rate.0;
                    self.usart.brr.write(|w| unsafe { w.bits(brr) });

                    // Reset other registers to disable advanced USART features
                    self.usart.cr2.reset();
                    self.usart.cr3.reset();
                }

                /// Starts listening for an interrupt event
                pub fn listen(&mut self, event: Event) {
                    match event {
                        Event::Rxne => {
                            self.usart.cr1.modify(|_, w| w.rxneie().set_bit())
                        },
                        Event::Txe => {
                            self.usart.cr1.modify(|_, w| w.txeie().set_bit())
                        },
                        Event::Idle => {
                            self.usart.cr1.modify(|_, w| w.idleie().set_bit())
                        },
                    }
                }

                /// Stop listening for an interrupt event
                pub fn unlisten(&mut self, event: Event) {
                    match event {
                        Event::Rxne => {
                            self.usart.cr1.modify(|_, w| w.rxneie().clear_bit())
                        },
                        Event::Txe => {
                            self.usart.cr1.modify(|_, w| w.txeie().clear_bit())
                        },
                        Event::Idle => {
                            self.usart.cr1.modify(|_, w| w.idleie().clear_bit())
                        },
                    }
                }

                /// Returns true if the line idle status is set
                pub fn is_idle(&self) -> bool {
                    self.usart.sr.read().idle().bit_is_set()
                }

                /// Returns true if the tx register is empty
                pub fn is_txe(&self) -> bool {
                    self.usart.sr.read().txe().bit_is_set()
                }

                /// Returns true if the rx register is not empty (and can be read)
                pub fn is_rx_not_empty(&self) -> bool {
                    self.usart.sr.read().rxne().bit_is_set()
                }

                /// Returns true if transmission is complete
                pub fn is_tx_complete(&self) -> bool {
                    self.usart.sr.read().tc().bit_is_set()
                }
            }
        )+
    }
}

#[cfg(any(
    feature = "py32f002a",
    feature = "py32f003",
    feature = "py32f030",
))]
usart! {
    USART1: (usart1, usart1tx, usart1rx, usart1en, apbenr2),
}

#[cfg(any(
    feature = "py32f003",
    feature = "py32f030",
))]
usart! {
    USART2: (usart2, usart2tx, usart2rx,usart2en, apbenr1),
}

impl<USART> embedded_hal::serial::Read<u8> for Rx<USART>
where
    USART: Deref<Target = SerialRegisterBlock>,
{
    type Error = Error;

    /// Tries to read a byte from the uart
    fn read(&mut self) -> nb::Result<u8, Error> {
        read(self.usart)
    }
}

impl<USART, TXPIN, RXPIN> embedded_hal::serial::Read<u8> for Serial<USART, TXPIN, RXPIN>
where
    USART: Deref<Target = SerialRegisterBlock>,
    RXPIN: RxPin<USART>,
{
    type Error = Error;

    /// Tries to read a byte from the uart
    fn read(&mut self) -> nb::Result<u8, Error> {
        read(&*self.usart)
    }
}

impl<USART> embedded_hal::serial::Write<u8> for Tx<USART>
where
    USART: Deref<Target = SerialRegisterBlock>,
{
    type Error = Infallible;

    /// Ensures that none of the previously written words are still buffered
    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        flush(self.usart)
    }

    /// Tries to write a byte to the uart
    /// Fails if the transmit buffer is full
    fn write(&mut self, byte: u8) -> nb::Result<(), Self::Error> {
        write(self.usart, byte)
    }
}

impl<USART, TXPIN, RXPIN> embedded_hal::serial::Write<u8> for Serial<USART, TXPIN, RXPIN>
where
    USART: Deref<Target = SerialRegisterBlock>,
    TXPIN: TxPin<USART>,
{
    type Error = Infallible;

    /// Ensures that none of the previously written words are still buffered
    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        flush(&*self.usart)
    }

    /// Tries to write a byte to the uart
    /// Fails if the transmit buffer is full
    fn write(&mut self, byte: u8) -> nb::Result<(), Self::Error> {
        write(&*self.usart, byte)
    }
}

impl<USART, TXPIN, RXPIN> Serial<USART, TXPIN, RXPIN>
where
    USART: Deref<Target = SerialRegisterBlock>,
{
    /// Splits the UART Peripheral in a Tx and an Rx part
    /// This is required for sending/receiving
    pub fn split(self) -> (Tx<USART>, Rx<USART>)
    where
        TXPIN: TxPin<USART>,
        RXPIN: RxPin<USART>,
    {
        (
            Tx {
                usart: &*self.usart,
                _instance: PhantomData,
            },
            Rx {
                usart: &*self.usart,
                _instance: PhantomData,
            },
        )
    }

    pub fn release(self) -> (USART, (TXPIN, RXPIN)) {
        (self.usart, self.pins)
    }
}

impl<USART> Write for Tx<USART>
where
    Tx<USART>: embedded_hal::serial::Write<u8>,
{
    fn write_str(&mut self, s: &str) -> Result {
        s.as_bytes()
            .iter()
            .try_for_each(|c| nb::block!(self.write(*c)))
            .map_err(|_| core::fmt::Error)
    }
}

impl<USART, TXPIN, RXPIN> Write for Serial<USART, TXPIN, RXPIN>
where
    USART: Deref<Target = SerialRegisterBlock>,
    TXPIN: TxPin<USART>,
{
    fn write_str(&mut self, s: &str) -> Result {
        s.as_bytes()
            .iter()
            .try_for_each(|c| nb::block!(self.write(*c)))
            .map_err(|_| core::fmt::Error)
    }
}

/// Ensures that none of the previously written words are still buffered
fn flush(usart: *const SerialRegisterBlock) -> nb::Result<(), Infallible> {
    // NOTE(unsafe) atomic read with no side effects
    let sr = unsafe { (*usart).sr.read() };

    if sr.tc().bit_is_set() {
        Ok(())
    } else {
        Err(nb::Error::WouldBlock)
    }
}

/// Tries to write a byte to the UART
/// Returns `Err(WouldBlock)` if the transmit buffer is full
fn write(usart: *const SerialRegisterBlock, byte: u8) -> nb::Result<(), Infallible> {
    // NOTE(unsafe) atomic read with no side effects
    let sr = unsafe { (*usart).sr.read() };

    if sr.txe().bit_is_set() {
        // NOTE(unsafe) atomic write to stateless register
        unsafe { (*usart).dr.write(|w| w.dr().bits(byte as u16)) }
        Ok(())
    } else {
        Err(nb::Error::WouldBlock)
    }
}

/// Tries to read a byte from the UART
fn read(usart: *const SerialRegisterBlock) -> nb::Result<u8, Error> {
    // NOTE(unsafe) atomic read with no side effects
    let sr = unsafe { (*usart).sr.read() };

    let dr = unsafe { (*usart).dr.read() };

    if sr.pe().bit_is_set() {
        Err(nb::Error::Other(Error::Parity))
    } else if sr.fe().bit_is_set() {
        Err(nb::Error::Other(Error::Framing))
    } else if sr.ne().bit_is_set() {
        Err(nb::Error::Other(Error::Noise))
    } else if sr.ore().bit_is_set() {
        Err(nb::Error::Other(Error::Overrun))
    } else if sr.rxne().bit_is_set() {
        Ok(dr.dr().bits() as u8)
    } else {
        Err(nb::Error::WouldBlock)
    }
}
