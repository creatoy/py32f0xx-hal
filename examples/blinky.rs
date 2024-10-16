#![no_main]
#![no_std]

use panic_halt as _;

use py32f0xx_hal as hal;

use crate::hal::{pac, prelude::*};

use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    if let Some(mut p) = pac::Peripherals::take() {
        let mut rcc = p.RCC.configure().sysclk(8.mhz()).freeze(&mut p.FLASH);

        let gpioa = p.GPIOA.split(&mut rcc);

        // (Re-)configure PA5 as output
        let mut led = cortex_m::interrupt::free(|cs| gpioa.pa5.into_push_pull_output(cs));

        let mut i = 0;
        loop {
            // Turn PA5 on a million times in a row
            for _ in 0..1_000_000 {
                led.set_high().ok();
            }
            // Then turn PA5 off a million times in a row
            for _ in 0..1_000_000 {
                led.set_low().ok();
            }

            i += 1;
        }
    }

    loop {
        continue;
    }
}
