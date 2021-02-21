#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use arduino_uno::{delay_ms, prelude::*, Delay};
use avr_device::interrupt;
use panic_halt as _;

mod temp;
mod tone;

use temp::Temp;
use tone::Tone;

const BOILING_POINT_C: f32 = 100.0;

#[arduino_uno::entry]
fn main() -> ! {
    let dp = arduino_uno::Peripherals::take().unwrap();

    let mut pins = arduino_uno::Pins::new(dp.PORTB, dp.PORTC, dp.PORTD);

    let mut led = pins.d13.into_output(&mut pins.ddr);
    led.set_low().void_unwrap();

    let mut tone = Tone::new(dp.TC0, pins.d2.into_output(&mut pins.ddr).downgrade());
    tone.sync_led(led);

    let mut delay = Delay::new();
    let mut temp = Temp::new(pins.d11.into_tri_state(&mut pins.ddr), &mut delay)
        .unwrap()
        .unwrap();

    unsafe {
        interrupt::enable();
    }

    tone.play(1200, 200);
    delay_ms(250);

    let mut hot = false;
    loop {
        if let Ok(c) = temp.read_c(&mut delay) {
            if c >= BOILING_POINT_C {
                hot = true;
            } else {
                hot = false;
            }
        }

        if hot {
            play_alarm(&mut tone);
        }
    }
}

fn play_alarm(tone: &mut Tone) {
    for _ in 0..4 {
        tone.play(1200, 200);
        delay_ms(250);
    }
}
