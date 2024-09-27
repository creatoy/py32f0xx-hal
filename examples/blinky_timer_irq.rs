#![no_main]
#![no_std]

use hal::rcc::HSIFreq;
use panic_halt as _;

use py32f0xx_hal as hal;

use crate::hal::{
    gpio::*,
    pac::{interrupt, Interrupt, Peripherals, TIM16},
    prelude::*,
    time::Hertz,
    timers::*,
};

use cortex_m_rt::entry;

use core::cell::RefCell;
use cortex_m::{interrupt::Mutex, peripheral::Peripherals as c_m_Peripherals};

// A type definition for the GPIO pin to be used for our LED
type LEDPIN = gpioa::PA5<Output<PushPull>>;

// Make LED pin globally available
static GLED: Mutex<RefCell<Option<LEDPIN>>> = Mutex::new(RefCell::new(None));

// Make timer interrupt registers globally available
static GINT: Mutex<RefCell<Option<Timer<TIM16>>>> = Mutex::new(RefCell::new(None));

// Define an interupt handler, i.e. function to call when interrupt occurs. Here if our external
// interrupt trips when the timer timed out
#[interrupt]
fn TIM16() {
    static mut LED: Option<LEDPIN> = None;
    static mut INT: Option<Timer<TIM16>> = None;

    let led = LED.get_or_insert_with(|| {
        cortex_m::interrupt::free(|cs| {
            // Move LED pin here, leaving a None in its place
            GLED.borrow(cs).replace(None).unwrap()
        })
    });

    let int = INT.get_or_insert_with(|| {
        cortex_m::interrupt::free(|cs| {
            // Move LED pin here, leaving a None in its place
            GINT.borrow(cs).replace(None).unwrap()
        })
    });

    led.toggle().ok();
    int.wait().ok();
}

#[entry]
fn main() -> ! {
    if let (Some(mut p), Some(cp)) = (Peripherals::take(), c_m_Peripherals::take()) {
        cortex_m::interrupt::free(move |cs| {
            let mut rcc = p
                .RCC
                .configure()
                .sysclk(24.mhz())
                .pclk(24.mhz())
                .freeze(&mut p.FLASH);

            let gpioa = p.GPIOA.split(&mut rcc);

            // (Re-)configure PA5 as output
            let led = gpioa.pa5.into_push_pull_output(cs);

            // Move the pin into our global storage
            *GLED.borrow(cs).borrow_mut() = Some(led);

            // Set up a timer expiring after 1s
            let mut timer = Timer::tim16(p.TIM16, Hertz(10), &mut rcc);

            // Generate an interrupt when the timer expires
            timer.listen(Event::TimeOut);

            // Move the timer into our global storage
            *GINT.borrow(cs).borrow_mut() = Some(timer);

            // Enable TIM16 IRQ, set prio 1 and clear any pending IRQs
            let mut nvic = cp.NVIC;
            unsafe {
                nvic.set_priority(Interrupt::TIM16, 1);
                cortex_m::peripheral::NVIC::unmask(Interrupt::TIM16);
            }
            cortex_m::peripheral::NVIC::unpend(Interrupt::TIM16);
        });
    }

    loop {
        continue;
    }
}
