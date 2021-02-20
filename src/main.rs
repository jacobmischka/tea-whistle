#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use arduino_uno::{delay_ms, prelude::*, Delay, Serial};
use avr_device::interrupt;
use one_wire_bus::OneWireError;
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
    let mut serial = Serial::new(
        dp.USART0,
        pins.d0,
        pins.d1.into_output(&mut pins.ddr),
        57600.into_baudrate(),
    );

    ufmt::uwriteln!(&mut serial, "Hello from Arduino!\r").void_unwrap();

    let mut delay = Delay::new();
    let mut temp = match Temp::new(pins.d10.into_tri_state(&mut pins.ddr), &mut delay) {
        Ok(Some(temp)) => temp,
        Ok(None) => {
            ufmt::uwriteln!(&mut serial, "No thermometer\r").void_unwrap();
            panic!()
        }
        Err(err) => {
            match err {
                OneWireError::BusNotHigh => {
                    ufmt::uwriteln!(&mut serial, "Bus not high\r").void_unwrap();
                }
                OneWireError::PinError(_) => {
                    ufmt::uwriteln!(&mut serial, "Pin error\r").void_unwrap();
                }
                OneWireError::UnexpectedResponse => {
                    ufmt::uwriteln!(&mut serial, "UnexpectedResponse\r").void_unwrap();
                }
                OneWireError::FamilyCodeMismatch => {
                    ufmt::uwriteln!(&mut serial, "Family code mismatch\r").void_unwrap();
                }
                OneWireError::CrcMismatch => {
                    ufmt::uwriteln!(&mut serial, "CRC mismatch\r").void_unwrap();
                }
                OneWireError::Timeout => {
                    ufmt::uwriteln!(&mut serial, "Timeout\r").void_unwrap();
                }
            }
            panic!()
        }
    };

    unsafe {
        interrupt::enable();
    }

    tone.play(261, 500);
    delay_ms(1000);

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
    tone.play(261, 2000);
    delay_ms(3000);
}
