#![no_main]
#![no_std]

use core::{cell::RefCell, fmt::Write};

use defmt_rtt as _;
use panic_halt as _;

use py32f0xx_hal as hal;

use crate::hal::{gpio, pac, prelude::*, rcc::HSIFreq};

use cortex_m::{interrupt::Mutex, peripheral::syst::SystClkSource::Core};
use cortex_m_rt::{entry, exception};
use defmt::info;

struct Shared {
    adc: hal::adc::Adc,
    tx: hal::serial::Tx<pac::USART1>,
    ain0: gpio::gpioa::PA0<gpio::Analog>,
    ain1: gpio::gpioa::PA1<gpio::Analog>,
}

static SHARED: Mutex<RefCell<Option<Shared>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    if let (Some(dp), Some(cp)) = (
        hal::pac::Peripherals::take(),
        cortex_m::peripheral::Peripherals::take(),
    ) {
        cortex_m::interrupt::free(move |cs| {
            let mut flash = dp.FLASH;
            let mut rcc = dp.RCC.configure().sysclk(24.mhz()).freeze(&mut flash);

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

            // USART1 at PA2 (TX) and PA3(RX)
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
            info!("\n\rThis ADC example will read various values using the ADC and print them out to the serial terminal\r\n");

            // Move all components under Mutex supervision
            *SHARED.borrow(cs).borrow_mut() = Some(Shared {
                adc,
                tx,
                ain0,
                ain1,
            });
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
            info!("Temperature {}.{}C\r", t / 100, t % 100);

            // Read volatage reference data from internal sensor using ADC
            let t = hal::adc::VRef::read_vdda(&mut shared.adc);
            writeln!(shared.tx, "Vdda {}mV\r", t).ok();
            info!("Vdda {}mV\r", t);

            // Read volatage data from external pin using ADC
            let v0 = hal::adc::Adc::read_abs_mv(&mut shared.adc, &mut shared.ain0);
            let v1 = hal::adc::Adc::read_abs_mv(&mut shared.adc, &mut shared.ain1);
            writeln!(shared.tx, "Ain0 {}mV Ain1 {}mV\r", v0, v1).ok();
            info!("Ain0 {}mV Ain1 {}mV\r", v0, v1);
        }
    });
}
