use arduino_uno::{
    hal::port::{mode, Pin},
    pac::TC0,
    prelude::*,
};
use avr_device::interrupt::{self, Mutex};

use core::cell::{Cell, RefCell, RefMut};

const CPU_FREQ: u32 = 16000;
const PRESCALERS: &[u16; 4] = &[1, 64, 256, 1024];

static TIMER: Mutex<RefCell<Option<TC0>>> = Mutex::new(RefCell::new(None));
static TONE_PIN: Mutex<RefCell<Option<Pin<mode::Output>>>> = Mutex::new(RefCell::new(None));
static TOGGLE_COUNTER: Mutex<Cell<u32>> = Mutex::new(Cell::new(0));

#[non_exhaustive]
pub struct Tone {}

impl Tone {
    pub fn new(tc0: TC0, pin: Pin<mode::Output>) -> Self {
        interrupt::free(|cs| {
            let timer_cell = TIMER.borrow(cs);
            timer_cell.replace(Some(tc0));

            let pin_cell = TONE_PIN.borrow(cs);
            pin_cell.replace(Some(pin));
        });

        Tone {}
    }

    /// frequency in hz, duration in ms
    pub fn play(&mut self, freq: u16, duration: u32) {
        let mut prescaler_index = 0;
        let mut ocr = get_ocr(freq, PRESCALERS[prescaler_index]);
        while ocr > 255 {
            prescaler_index += 1;
            ocr = get_ocr(freq, PRESCALERS[prescaler_index]);
        }

        interrupt::free(|cs| {
            // initialize timer
            let timer_cell = TIMER.borrow(cs);
            let ref_mut = timer_cell.borrow_mut();
            RefMut::map(ref_mut, |opt| {
                if let Some(tc0) = opt {
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
                opt
            });

            // set counter
            let counter_cell = TOGGLE_COUNTER.borrow(cs);
            counter_cell.set(2 * freq as u32 * duration / 1000);
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
            let timer_cell = TIMER.borrow(cs);
            let ref_mut = timer_cell.borrow_mut();
            RefMut::map(ref_mut, |opt| {
                if let Some(tc0) = opt {
                    is_playing = tc0.timsk0.read().ocie0a().bit_is_set();
                }
                opt
            });
        });

        is_playing
    }
}

fn get_ocr(freq: u16, prescaler: u16) -> u32 {
    CPU_FREQ / freq as u32 / 2 / prescaler as u32 - 1
}

fn stop_tone(cs: &interrupt::CriticalSection) {
    // disable interrupt
    let timer_cell = TIMER.borrow(cs);
    let ref_mut = timer_cell.borrow_mut();
    RefMut::map(ref_mut, |opt| {
        if let Some(tc0) = opt {
            tc0.timsk0.write(|w| w.ocie0a().clear_bit());
        }
        opt
    });

    // set pin low
    let pin_cell = TONE_PIN.borrow(cs);
    let ref_mut = pin_cell.borrow_mut();
    RefMut::map(ref_mut, |opt| {
        if let Some(pin) = opt {
            pin.set_high().void_unwrap();
        }
        opt
    });
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
            let pin_cell = TONE_PIN.borrow(cs);
            let ref_mut = pin_cell.borrow_mut();
            RefMut::map(ref_mut, |opt| {
                if let Some(pin) = opt {
                    pin.toggle().void_unwrap();
                }
                opt
            });

            // decrement counter
            counter_cell.set(counter - 1);
        }
    });
}
