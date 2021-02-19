use arduino_uno::hal::port::{mode, Pin};
use ds18b20::Ds18b20;
use embedded_hal::blocking::delay::DelayUs;
use one_wire_bus::{OneWire, OneWireResult};

pub struct Temp {
    one_wire_bus: OneWire<Pin<mode::TriState>>,
    sensor: Ds18b20,
}

impl Temp {
    fn new<E>(
        p: Pin<mode::TriState>,
        delay: &mut impl DelayUs<u16>,
    ) -> OneWireResult<Option<Self>, E> {
        let mut one_wire_bus = OneWire::new(p)?;
        for device_address in one_wire_bus.devices(false, delay) {
            if let Ok(device_address) = device_address {
                if device_address.family_code() == ds18b20::FAMILY_CODE {
                    let sensor = Ds18b20::new(device_address)?;

                    return Ok(Some(Temp {
                        one_wire_bus,
                        sensor,
                    }));
                }
            }
        }

        Ok(None)
    }
}
