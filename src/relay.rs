use esp_idf_svc::hal::{
    gpio::{Output, OutputPin, Pin, PinDriver},
    peripheral::Peripheral,
    sys::EspError,
};

/// Relay driver
pub struct Driver<'a, T: Pin>(PinDriver<'a, T, Output>);

impl<'a, T: OutputPin> Driver<'a, T> {
    pub fn new(pin: impl Peripheral<P = T> + 'a) -> Result<Self, EspError> {
        Ok(Self(PinDriver::output(pin)?))
    }
}
