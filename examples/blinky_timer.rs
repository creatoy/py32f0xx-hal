#![no_main]
#![no_std]

use panic_halt as _;

use py32f0xx_hal as hal;

use crate::hal::{pac, prelude::*, time::Hertz, timers::*};

use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    if let Some(mut p) = pac::Peripherals::take() {
        let mut rcc = p.RCC.configure().sysclk(8.mhz()).freeze(&mut p.FLASH);

        let gpioa = p.GPIOA.split(&mut rcc);

        // (Re-)configure PA5 as output
        let mut led = cortex_m::interrupt::free(move |cs| gpioa.pa5.into_push_pull_output(cs));

        // Set up a timer expiring after 1s
        let mut timer = Timer::tim1(p.TIM1, Hertz(5), &mut rcc);

        loop {
            led.toggle().ok();

            // Wait for the timer to expire
            nb::block!(timer.wait()).ok();
        }
    }

    loop {
        continue;
    }
}
