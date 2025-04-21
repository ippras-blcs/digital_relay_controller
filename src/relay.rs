use esp_idf_svc::hal::{gpio::IOPin, peripheral::Peripheral, rmt::RmtChannel};
use log::{info, trace};
use std::{collections::BTreeMap, ops::Range};
use thiserror::Error;
use tokio::{
    spawn,
    sync::{
        mpsc::{self, Sender},
        oneshot::Sender as OneshotSender,
    },
};
use tokio_modbus::prelude::ExceptionCode;

pub(crate) type Request = (
    Range<usize>,
    OneshotSender<Result<BTreeMap<u64, f32>, Error>>,
);

#[derive(Clone, Copy, Debug, Default)]
pub enum State {
    On,
    #[default]
    Off,
}

pub(super) fn start(
    pin: impl Peripheral<P = impl IOPin> + 'static,
    channel: impl Peripheral<P = impl RmtChannel> + 'static,
) -> Result<Sender<Request>> {
    info!("Initialize relay reader");
    let mut driver = Ds18b20Driver::new(pin, channel)?;
    info!("Relay driver initialized");

    let (sender, mut receiver) = mpsc::channel::<Request>(9);
    info!("Spawn relay reader");
    spawn(async move {
        while let Some((indices, sender)) = receiver.recv().await {
            trace!("Read relay {indices:?}");
            let _ = sender.send((|| {
                trace!("Send relay {indices:?}");
                if indices.end > addresses.len() {
                    return Err(Error::InvalidIndex {
                        received: indices,
                        expected: 0..addresses.len(),
                    });
                }
                let mut temperatures = BTreeMap::new();
                driver.initialization()?.skip_rom()?.convert_temperature()?;
                for index in indices {
                    let address = &addresses[index];
                    let temperature = driver
                        .initialization()?
                        .match_rom(address)?
                        .read_scratchpad()?
                        .temperature;
                    trace!("{address:x?}: {temperature}");
                    temperatures.insert(address.address(), temperature);
                }
                Ok(temperatures)
            })());
        }
    });
    Ok(sender)
}

/// Result
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Error
#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid index {{ received: {received:?}, expected: {expected:?} }}")]
    InvalidIndex {
        received: Range<usize>,
        expected: Range<usize>,
    },
}

impl From<Error> for ExceptionCode {
    fn from(value: Error) -> Self {
        match value {
            Error::InvalidIndex { .. } => ExceptionCode::IllegalDataAddress,
            // Error::Internal(_) => ExceptionCode::ServerDeviceFailure,
        }
    }
}
