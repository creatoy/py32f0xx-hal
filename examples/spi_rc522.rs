#![no_main]
#![no_std]

use defmt_rtt as _;
use panic_probe as _;

use py32f0xx_hal as hal;

use crate::hal::{
    delay::Delay,
    pac,
    prelude::*,
    serial::Serial,
    spi::Spi,
    spi::{Mode, Phase, Polarity},
    time::Hertz,
    timers::Timer,
};

use nb::block;

use cortex_m_rt::entry;

use defmt::{error, info};

use mfrc522::comm::{eh02::spi::SpiInterface, Interface};
use mfrc522::error::Error;
use mfrc522::{Initialized, Mfrc522, Uid};

/// A basic serial to spi example
///
/// If you connect MOSI & MISO pins together, you'll see all characters
/// that you typed in your serial terminal echoed back
///
/// If you connect MISO to GND, you'll see nothing coming back
#[entry]
fn main() -> ! {
    const MODE: Mode = Mode {
        polarity: Polarity::IdleHigh,
        phase: Phase::CaptureOnSecondTransition,
    };

    if let (Some(p), Some(cp)) = (
        pac::Peripherals::take(),
        cortex_m::peripheral::Peripherals::take(),
    ) {
        let mut flash = p.FLASH;
        let mut rcc = p.RCC.configure().freeze(&mut flash);

        let mut delay = Delay::new(cp.SYST, &rcc);

        let gpioa = p.GPIOA.split(&mut rcc);

        let (sck, miso, mosi, mut nss, mut rst) = cortex_m::interrupt::free(move |cs| {
            (
                // SPI pins
                gpioa.pa5.into_alternate_af0(cs),
                gpioa.pa6.into_alternate_af0(cs),
                gpioa.pa7.into_alternate_af0(cs),
                // Aux pins
                gpioa.pa4.into_push_pull_output(cs),
                gpioa.pa1.into_push_pull_output(cs),
            )
        });
        rst.set_low();

        // Configure SPI with 1MHz rate
        let mut spi = Spi::spi1(p.SPI1, (sck, miso, mosi), MODE, 1.mhz(), &mut rcc);
        let itf = SpiInterface::new(spi).with_nss(nss).with_delay(|| {
            delay.delay_ms(1_u16);
        });
        rst.set_high();

        let mut mfrc522 = Mfrc522::new(itf).init().unwrap();

        let ver = mfrc522.version().unwrap();
        info!("MFRC522 version: 0x{:02x}", ver);
        assert!(ver == 0x91 || ver == 0x92);

        let mut timer = Timer::tim1(p.TIM1, Hertz(1), &mut rcc);

        loop {
            info!("Waiting for card...");
            match mfrc522.reqa() {
                Ok(atqa) => {
                    if let Ok(uid) = mfrc522.select(&atqa) {
                        info!("Selected card, UID: {:02X}", uid.as_bytes());
                    } else {
                        error!("Failed to select card");
                    }
                }
                Err(e) => error!("Error when requesting ATQA!"),
            }

            nb::block!(timer.wait()).unwrap();
        }
    }

    loop {
        continue;
    }
}
