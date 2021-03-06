use avr_device::interrupt;
use ds18b20::{Ds18b20, Resolution, SensorData};
use embedded_hal::{
    blocking::delay::{DelayMs, DelayUs},
    digital::v2::{InputPin, OutputPin},
};
use one_wire_bus::{OneWire, OneWireResult};

pub struct Temp<P> {
    one_wire_bus: OneWire<P>,
    sensor: Ds18b20,
    resolution: Resolution,
}

#[allow(dead_code)]
impl<E, P: OutputPin<Error = E> + InputPin<Error = E>> Temp<P> {
    pub fn new(
        p: P,
        delay: &mut (impl DelayUs<u16> + DelayMs<u16>),
    ) -> OneWireResult<Option<Self>, E> {
        let mut one_wire_bus = OneWire::new(p)?;

        for device_address in one_wire_bus.devices(false, delay) {
            let device_address = device_address?;
            if device_address.family_code() == ds18b20::FAMILY_CODE {
                let sensor = Ds18b20::new(device_address)?;
                let SensorData { resolution, .. } = sensor.read_data(&mut one_wire_bus, delay)?;

                return Ok(Some(Temp {
                    one_wire_bus,
                    sensor,
                    resolution,
                }));
            }
        }

        Ok(None)
    }

    pub fn c_to_f(c: f32) -> f32 {
        c * 1.8 + 32.0
    }

    pub fn f_to_c(f: f32) -> f32 {
        (f - 32.0) / 1.8
    }

    pub fn read_c(
        &mut self,
        delay: &mut (impl DelayUs<u16> + DelayMs<u16>),
    ) -> OneWireResult<f32, E> {
        interrupt::free(|_| {
            self.sensor
                .start_temp_measurement(&mut self.one_wire_bus, delay)?;
            self.resolution.delay_for_measurement_time(delay);
            self.sensor
                .read_data(&mut self.one_wire_bus, delay)
                .map(|data| data.temperature)
        })
    }

    pub fn read_f(
        &mut self,
        delay: &mut (impl DelayUs<u16> + DelayMs<u16>),
    ) -> OneWireResult<f32, E> {
        self.read_c(delay).map(|c| <Temp<P>>::c_to_f(c))
    }
}

#[cfg(test)]
mod test {
    use super::Temp;

    #[test]
    fn conversions_round_trip() {
        for i in 0.0..100.0 {
            assert_eq!(Temp::c_to_f(Temp::f_to_c(i)), i);
        }
    }
}
