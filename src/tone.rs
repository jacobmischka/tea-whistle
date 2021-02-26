use arduino_uno::{
    hal::port::{mode, portb::PB5, Pin},
    pac::TC0,
    prelude::*,
};
use avr_device::interrupt::{self, Mutex};

use core::cell::{Cell, RefCell};

const CPU_FREQ: u32 = 16_000_000;
const PRESCALERS: &[u16; 5] = &[1, 8, 64, 256, 1024];

static LED: Mutex<RefCell<Option<PB5<mode::Output>>>> = Mutex::new(RefCell::new(None));
static TIMER: Mutex<RefCell<Option<TC0>>> = Mutex::new(RefCell::new(None));
static TONE_PIN: Mutex<RefCell<Option<Pin<mode::Output>>>> = Mutex::new(RefCell::new(None));
static TOGGLE_COUNTER: Mutex<Cell<u32>> = Mutex::new(Cell::new(0));

#[non_exhaustive]
pub struct Tone {}

#[allow(dead_code)]
impl Tone {
    pub fn new(tc0: TC0, pin: Pin<mode::Output>) -> Self {
        interrupt::free(|cs| {
            TIMER.borrow(cs).replace(Some(tc0));
            TONE_PIN.borrow(cs).replace(Some(pin));
        });

        Tone {}
    }

    pub fn sync_led(&mut self, led: PB5<mode::Output>) {
        interrupt::free(|cs| {
            LED.borrow(cs).replace(Some(led));
        });
    }

    /// frequency in hz, duration in ms
    pub fn play(&mut self, freq: u16, duration: u32) {
        if self.is_playing() {
            return;
        }

        let mut prescaler_index = 0;
        let mut ocr = get_ocr(freq, PRESCALERS[prescaler_index]);
        while ocr > 255 {
            prescaler_index += 1;
            ocr = get_ocr(freq, PRESCALERS[prescaler_index]);
        }

        interrupt::free(|cs| {
            // initialize timer
            let mut timer = TIMER.borrow(cs).borrow_mut();
            if let Some(tc0) = timer.as_mut() {
                tc0.tccr0a.write(|w| w.wgm0().ctc());
                tc0.ocr0a.write(|w| unsafe { w.bits(ocr as u8) });
                tc0.tccr0b.write(|w| match PRESCALERS[prescaler_index] {
                    1 => w.cs0().direct(),
                    8 => w.cs0().prescale_8(),
                    64 => w.cs0().prescale_64(),
                    256 => w.cs0().prescale_256(),
                    1024 => w.cs0().prescale_1024(),
                    _ => panic!(),
                });
                tc0.timsk0.write(|w| w.ocie0a().set_bit());
            }

            // set counter
            TOGGLE_COUNTER
                .borrow(cs)
                .set(2 * freq as u32 * duration / 1000);

            // set led
            set_led(cs, true);
        });
    }

    pub fn stop(&mut self) {
        interrupt::free(|cs| {
            stop_tone(cs);
        });
    }

    pub fn is_playing(&self) -> bool {
        let mut is_playing = false;

        // check if interrupt is enabled
        interrupt::free(|cs| {
            let mut timer = TIMER.borrow(cs).borrow_mut();
            if let Some(tc0) = timer.as_mut() {
                is_playing = tc0.timsk0.read().ocie0a().bit_is_set();
            }
        });

        is_playing
    }
}

impl Drop for Tone {
    fn drop(&mut self) {
        interrupt::free(|cs| {
            LED.borrow(cs).replace(None);
            TIMER.borrow(cs).replace(None);
            TONE_PIN.borrow(cs).replace(None);
            TOGGLE_COUNTER.borrow(cs).replace(0);
        });
    }
}

fn get_ocr(freq: u16, prescaler: u16) -> u32 {
    CPU_FREQ / freq as u32 / 2 / prescaler as u32 - 1
}

fn stop_tone(cs: &interrupt::CriticalSection) {
    // disable interrupt
    let mut timer = TIMER.borrow(cs).borrow_mut();
    if let Some(tc0) = timer.as_mut() {
        tc0.timsk0.write(|w| w.ocie0a().clear_bit());
    }

    // set pin low
    let mut pin = TONE_PIN.borrow(cs).borrow_mut();
    if let Some(pin) = pin.as_mut() {
        pin.set_low().void_unwrap();
    }

    set_led(cs, false);
}

fn set_led(cs: &interrupt::CriticalSection, lit: bool) -> Option<()> {
    LED.borrow(cs).borrow_mut().as_mut().map(|led| {
        if lit {
            led.set_high().void_unwrap();
        } else {
            led.set_low().void_unwrap();
        }
    })
}

#[avr_device::interrupt(atmega328p)]
fn TIMER0_COMPA() {
    interrupt::free(|cs| {
        let counter_cell = TOGGLE_COUNTER.borrow(cs);
        let counter = counter_cell.get();

        if counter == 0 {
            stop_tone(cs);
        } else {
            // toggle pin
            let mut pin = TONE_PIN.borrow(cs).borrow_mut();
            if let Some(pin) = pin.as_mut() {
                pin.toggle().void_unwrap();
            }

            // decrement counter
            counter_cell.set(counter - 1);
        }
    });
}
