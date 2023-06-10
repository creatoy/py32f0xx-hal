#![no_main]
#![no_std]

use core::fmt::Write;

use panic_halt as _;

use py32f0xx_hal as hal;

use crate::hal::{
    pac,
    prelude::*,
    rcc::{HSIFreq, MCOSrc, MCODiv},
    serial::Serial
};

use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    if let Some(p) = pac::Peripherals::take() {
        let mut flash = p.FLASH;
        let mut rcc = p.RCC.configure()
            .hsi(HSIFreq::Freq24mhz)
            // .hse(24.mhz(), hal::rcc::HSEBypassMode::NotBypassed)
            .sysclk(48.mhz()).freeze(&mut flash);

        rcc.configure_mco(MCOSrc::Sysclk, MCODiv::NotDivided);

        let gpioa = p.GPIOA.split(&mut rcc);

        let (tx, rx) = cortex_m::interrupt::free(move |cs| {
            gpioa.pa1.into_alternate_af15(cs);
            (
                gpioa.pa2.into_alternate_af1(cs),
                gpioa.pa3.into_alternate_af1(cs),
            )
        });

        let mut serial = Serial::usart1(p.USART1, (tx, rx), 115_200.bps(), &mut rcc);
        serial.write_str("Input any key:\n").ok();

        loop {
            // Wait for reception of a single byte
            let received = nb::block!(serial.read()).unwrap();

            // Send back previously received byte and wait for completion
            nb::block!(serial.write(received)).ok();
        }
    }

    loop {
        continue;
    }
}
