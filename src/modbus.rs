// use crate::relay::Request as LedRequest;
use crate::{RelayDriver, led::Request as LedRequest};
use anyhow::Result;
use esp_idf_hal::{
    gpio::OutputPin,
    peripheral::{Peripheral, PeripheralRef},
};
use log::{error, info, warn};
use std::{net::SocketAddr, sync::LazyLock, time::Duration};
use tokio::{
    net::TcpListener,
    sync::{mpsc::Sender, oneshot},
};
use tokio_modbus::{
    prelude::*,
    server::{
        Service,
        tcp::{Server, accept_tcp_connection},
    },
};

const INPUT_REGISTER_SIZE: usize = 6;

static SOCKET_ADDR: LazyLock<SocketAddr> = LazyLock::new(|| "0.0.0.0:5502".parse().unwrap());

pub(super) async fn run(
    pin1: impl Peripheral<P = impl OutputPin> + 'static,
    pin2: impl Peripheral<P = impl OutputPin> + 'static,
    led_sender: Sender<LedRequest>,
) -> Result<()> {
    let driver1 = RelayDriver::new(pin1)?;
    let driver2 = RelayDriver::new(pin2)?;
    let server = Server::new(TcpListener::bind(*SOCKET_ADDR).await?);
    let new_service = |_socket_addr| Ok(Some(RelayService::new(led_sender.clone())));
    let on_connected = |stream, socket_addr| async move {
        accept_tcp_connection(stream, socket_addr, new_service)
    };
    let on_process_error = |error| error!("{error}");
    server.serve(&on_connected, on_process_error).await?;
    Ok(())
}

/// Relay service
struct RelayService {
    led_sender: Sender<LedRequest>,
}

impl RelayService {
    fn new(led_sender: Sender<LedRequest>) -> Self {
        Self { led_sender }
    }
}

impl Service for RelayService {
    type Request = Request<'static>;
    type Response = Response;
    type Exception = ExceptionCode;
    type Future = impl Future<Output = Result<Self::Response, Self::Exception>>;

    fn call(&self, request: Self::Request) -> Self::Future {
        info!("Modbus request: {request:?}");
        let led_sender = self.led_sender.clone();
        async move {
            let _ = led_sender.send(Ok(Duration::from_millis(100))).await;
            match request {
                Request::ReadCoils(address, count) => {
                    if address > 1 || address + count > 2 {
                        error!("IllegalAddress {{ address: {address}, count: {count} }}");
                        return Err(ExceptionCode::IllegalDataAddress);
                    }
                    if count == 0 {
                        return Ok(Response::ReadCoils(vec![]));
                    }
                    Ok(Response::ReadCoils(vec![false]))
                }
                Request::WriteSingleCoil(address, value) => {
                    if address > 1 {
                        error!("IllegalAddress {{ address: {address} }}");
                        return Err(ExceptionCode::IllegalDataAddress);
                    }
                    Ok(Response::WriteSingleCoil(address, value))
                }
                Request::WriteMultipleCoils(address, values) => {
                    let count = values.len() as u16;
                    if address > 1 || address + count > 2 {
                        error!("IllegalAddress {{ address: {address} }}");
                        return Err(ExceptionCode::IllegalDataAddress);
                    }
                    if count == 0 {
                        return Ok(Response::ReadCoils(vec![]));
                    }
                    Ok(Response::WriteMultipleCoils(address, values.len() as _))
                }
                // Request::ReadInputRegisters(address, count) => {
                //     let address = address as usize;
                //     let count = count as usize;
                //     if address % INPUT_REGISTER_SIZE != 0 || count % INPUT_REGISTER_SIZE != 0 {
                //         error!("IllegalAddress {{ address: {address}, count: {count} }}");
                //         return Err(ExceptionCode::IllegalDataAddress);
                //     }
                //     let start = address / INPUT_REGISTER_SIZE;
                //     let end = start + count / INPUT_REGISTER_SIZE;
                //     let (sender, receiver) = oneshot::channel();
                //     if let Err(error) = temperature_sender.send((start..end, sender)).await {
                //         error!("{error:?}");
                //         return Err(ExceptionCode::ServerDeviceFailure);
                //     };
                //     let input_registers: Vec<_> = match receiver.await {
                //         Ok(Ok(temperatures)) => temperatures
                //             .into_iter()
                //             .flat_map(|(address, temperature)| {
                //                 let address = address.to_be_bytes();
                //                 let temperature = temperature.to_be_bytes();
                //                 [
                //                     u16::from_be_bytes([address[0], address[1]]),
                //                     u16::from_be_bytes([address[2], address[3]]),
                //                     u16::from_be_bytes([address[4], address[5]]),
                //                     u16::from_be_bytes([address[6], address[7]]),
                //                     u16::from_be_bytes([temperature[0], temperature[1]]),
                //                     u16::from_be_bytes([temperature[2], temperature[3]]),
                //                 ]
                //             })
                //             .collect(),
                //         Ok(Err(error)) => {
                //             error!("{error:?}");
                //             return Err(error.into());
                //         }
                //         Err(error) => {
                //             error!("{error:?}");
                //             return Err(ExceptionCode::ServerDeviceFailure);
                //         }
                //     };
                //     Ok(Response::ReadInputRegisters(
                //         input_registers[address..count].to_vec(),
                //     ))
                // }
                _ => {
                    let _ = led_sender.send(Err(Duration::from_millis(100))).await;
                    Err(ExceptionCode::IllegalFunction)
                }
            }
        }
    }
}
