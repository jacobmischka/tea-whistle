#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use arduino_uno::prelude::*;
use avr_device::interrupt;
use panic_halt as _;

mod tone;

use tone::Tone;

#[arduino_uno::entry]
fn main() -> ! {
    let dp = arduino_uno::Peripherals::take().unwrap();

    let mut pins = arduino_uno::Pins::new(dp.PORTB, dp.PORTC, dp.PORTD);

    // Digital pin 13 is also connected to an onboard LED marked "L"
    let mut led = pins.d13.into_output(&mut pins.ddr);
    led.set_low().void_unwrap();

    let mut tone = Tone::new(dp.TC0, pins.d2.into_output(&mut pins.ddr).downgrade());
    tone.sync_led(led);

    unsafe {
        interrupt::enable();
    }

    loop {
        tone.play(261, 2000);
        arduino_uno::delay_ms(4000);
    }
}
