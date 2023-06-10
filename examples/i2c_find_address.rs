#![no_main]
#![no_std]

use panic_halt as _;

use rtt_target::{rprintln, rtt_init_default};

use py32f0xx_hal as hal;

use crate::hal::{i2c::I2c, pac, prelude::*};

use cortex_m_rt::entry;

/* Example meant for py32f030xc MCUs with i2c devices connected on PB7 and PB8 */

#[entry]
fn main() -> ! {
    let channels = rtt_init_default!();
    rtt_target::set_print_channel(channels.up.0);

    if let Some(p) = pac::Peripherals::take() {
        cortex_m::interrupt::free(move |cs| {
            let mut flash = p.FLASH;
            let mut rcc = p.RCC.configure().freeze(&mut flash);

            let gpioa = p.GPIOA.split(&mut rcc);

            // Configure pins for I2C
            let scl = gpioa.pa3.into_alternate_af12(cs);
            let sda = gpioa.pa2.into_alternate_af12(cs);

            // Configure I2C with 100kHz rate
            let mut i2c = I2c::i2c(p.I2C, (scl, sda), 100.khz(), &mut rcc);

            let mut devices = 0;
            // I2C addresses are 7-bit wide, covering the 0-127 range
            for add in 0..=127 {
                // The write method sends the specified address and checks for acknowledgement;
                // if no ack is given by the slave device the result is Err(), otherwise Ok()
                // Since we only care for an acknowledgement the data sent can be empty
                if i2c.write(add, &[]).is_ok() {
                    devices += 1;
                }
            }

            // Here the variable "_devices" counts how many i2c addresses were a hit
            rprintln!("{} devices find.\r\n", devices);
        });
    }

    loop {
        continue;
    }
}
