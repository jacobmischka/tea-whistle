use arduino_uno::{
    hal::port::{mode, Pin},
    pac::TC1,
    prelude::*,
};
use avr_device::interrupt::{self, Mutex};

use core::cell::{Cell, RefCell, RefMut};

pub struct Tone {}

impl Tone {
    fn new(tc1: TC1, pin: Pin<mode::Output>) -> Self {
        interrupt::free(|cs| {
            let timer_cell = TIMER.borrow(cs);
            timer_cell.replace(Some(tc1));

            let pin_cell = TONE_PIN.borrow(cs);
            pin_cell.replace(Some(pin));
        });

        Tone {}
    }

    fn play(&mut self, freq: u16, duration: u32) {
        todo!()
    }
}

static TIMER: Mutex<RefCell<Option<TC1>>> = Mutex::new(RefCell::new(None));
static TONE_PIN: Mutex<RefCell<Option<Pin<mode::Output>>>> = Mutex::new(RefCell::new(None));
static TOGGLE_COUNTER: Mutex<Cell<u32>> = Mutex::new(Cell::new(0));

#[avr_device::interrupt(atmega328p)]
fn TIMER0_COMPA() {
    interrupt::free(|cs| {
        let counter_cell = TOGGLE_COUNTER.borrow(cs);
        let counter = counter_cell.get();
        if counter == 0 {
            // disable interrupt
            let timer_cell = TIMER.borrow(cs);
            let ref_mut = timer_cell.borrow_mut();
            RefMut::map(ref_mut, |opt| {
                if let Some(tc1) = opt {
                    tc1.timsk1.write(|w| w.ocie1a().clear_bit());
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

            // decrement timer
            let counter_cell = TOGGLE_COUNTER.borrow(cs);
            let counter = counter_cell.get();
            counter_cell.set(counter - 1);
        }
    });
}
