#![no_main]
#![no_std]

use panic_halt as _;

use rtt_target::{rprintln, rtt_init_default};

use py32f0xx_hal as hal;

use crate::hal::{pac, prelude::*, gpio, rcc::HSIFreq};

use cortex_m::{interrupt::Mutex, peripheral::syst::SystClkSource::Core};
use cortex_m_rt::{entry, exception};

use core::{cell::RefCell, fmt::Write};

struct Shared {
    adc: hal::adc::Adc,
    tx: hal::serial::Tx<pac::USART1>,
    ain1: gpio::gpioa::PA1<gpio::Analog>,
}

static SHARED: Mutex<RefCell<Option<Shared>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let channels = rtt_init_default!();
    rtt_target::set_print_channel(channels.up.0);

    if let (Some(dp), Some(cp)) = (
        hal::pac::Peripherals::take(),
        cortex_m::peripheral::Peripherals::take(),
    ) {
        cortex_m::interrupt::free(move |cs| {
            let mut flash = dp.FLASH;
            let mut rcc = dp.RCC.configure()
                .hsi(HSIFreq::Freq24mhz)
                .sysclk(24.mhz()).freeze(&mut flash);

            let gpioa = dp.GPIOA.split(&mut rcc);

            let mut syst = cp.SYST;

            // Set source for SysTick counter, here full operating frequency (== 24MHz)
            syst.set_clock_source(Core);

            // Set reload value, i.e. timer delay 24 MHz/counts
            syst.set_reload(24_000_000 - 1);

            // Start SysTick counter
            syst.enable_counter();

            // Start SysTick interrupt generation
            syst.enable_interrupt();

            // USART1 at PA9 (TX) and PA10(RX)
            let tx = gpioa.pa2.into_alternate_af1(cs);
            let rx = gpioa.pa3.into_alternate_af1(cs);

            // Initialiase UART
            let (mut tx, _) =
                hal::serial::Serial::usart1(dp.USART1, (tx, rx), 115_200.bps(), &mut rcc).split();

            // Initialise ADC
            let adc = hal::adc::Adc::new(dp.ADC, &mut rcc, hal::adc::AdcClockMode::default());

            let ain0 = gpioa.pa0.into_analog(cs); // ADC_IN0
            let ain1 = gpioa.pa1.into_analog(cs); // ADC_IN1

            // Output a friendly greeting
            tx.write_str("\n\rThis ADC example will read various values using the ADC and print them out to the serial terminal\r\n").ok();

            rprintln!("\n\rThis ADC example will read various values using the ADC and print them out to the serial terminal\r\n");

            // Move all components under Mutex supervision
            *SHARED.borrow(cs).borrow_mut() = Some(Shared { adc, tx, ain1 });
        });
    }

    loop {
        continue;
    }
}

#[exception]
fn SysTick() {
    use core::ops::DerefMut;

    // Enter critical section
    cortex_m::interrupt::free(|cs| {
        // Get access to the Mutex protected shared data
        if let Some(ref mut shared) = SHARED.borrow(cs).borrow_mut().deref_mut() {
            // Read temperature data from internal sensor using ADC
            let t = hal::adc::VTemp::read(&mut shared.adc, None);
            writeln!(shared.tx, "Temperature {}.{}C\r", t / 100, t % 100).ok();
            rprintln!("Temperature {}.{}C\r", t / 100, t % 100);

            // Read volatage reference data from internal sensor using ADC
            let t = hal::adc::VRef::read_vdda(&mut shared.adc);
            writeln!(shared.tx, "Vdda {}mV\r", t).ok();
            rprintln!("Vdda {}mV\r", t);

            // Read volatage reference data from internal sensor using ADC
            let t = hal::adc::Adc::read_abs_mv(&mut shared.adc, &mut shared.ain1);
            writeln!(shared.tx, "Ain1 {}mV\r", t).ok();
            rprintln!("Ain1 {}mV\r", t);
        }
    });
}
