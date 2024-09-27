#![deny(unsafe_code)]
#![no_main]
#![no_std]

// Halt on panic
use panic_halt as _;

use cortex_m_rt::entry;

use py32f0xx_hal as hal;

use hal::{pac, prelude::*, pwm};

#[entry]
fn main() -> ! {
    if let Some(mut dp) = pac::Peripherals::take() {
        // Set up the system clock.
        let mut rcc = dp.RCC.configure().sysclk(8.mhz()).freeze(&mut dp.FLASH);

        let gpioa = dp.GPIOA.split(&mut rcc);
        let channels = cortex_m::interrupt::free(move |cs| {
            (
                gpioa.pa8.into_alternate_af2(cs), // on TIM1_CH1
                gpioa.pa9.into_alternate_af2(cs), // on TIM1_CH2
            )
        });

        let pwm = pwm::tim1(dp.TIM1, channels, &mut rcc, 20u32.khz());
        let (mut ch1, mut ch2) = pwm;
        let max_duty = ch1.get_max_duty();
        ch1.set_duty(max_duty / 2);
        ch1.enable();
        ch2.set_duty(max_duty * 9 / 10);
        ch2.enable();
    }

    loop {
        cortex_m::asm::nop();
    }
}
