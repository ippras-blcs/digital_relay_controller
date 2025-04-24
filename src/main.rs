#![feature(impl_trait_in_assoc_type)]

use self::{relay::Driver as RelayDriver, wifi::connect};
use anyhow::{Result, bail};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{
        delay::FreeRtos,
        gpio::{PinDriver, Pull},
        prelude::Peripherals,
        reset::restart,
        rmt::{FixedLengthSignal, PinState, Pulse, TxRmtDriver, config::TransmitConfig},
    },
    io::vfs::MountedEventfs,
    log::EspLogger,
    nvs::EspDefaultNvsPartition,
    sys::link_patches,
    timer::EspTaskTimerService,
    wifi::WifiEvent,
};
use log::{error, info, warn};
use tokio::{
    runtime::Builder,
    spawn,
    time::{Duration, sleep},
};

fn main() -> Result<()> {
    link_patches();
    EspLogger::initialize_default();
    let _mounted_eventfs = MountedEventfs::mount(5)?;
    info!("System initialized");
    if let Err(error) = Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(run())
    {
        error!("{error:?}");
    } else {
        info!("`main()` finished, restarting");
    }
    restart();
}

async fn run() -> Result<()> {
    let event_loop = EspSystemEventLoop::take()?;
    let timer = EspTaskTimerService::new()?;
    let peripherals = Peripherals::take()?;
    let nvs = EspDefaultNvsPartition::take()?;
    // Initialize the network stack, this must be done before starting the server
    let mut wifi = connect(peripherals.modem, event_loop.clone(), timer, Some(nvs)).await?;
    let _subscription = event_loop.subscribe::<WifiEvent, _>(move |event| {
        info!("Got event: {event:?}");
        if let WifiEvent::StaDisconnected(_) = event {
            if let Err(error) = wifi.connect() {
                warn!("Wifi connect failed: {error}");
            }
        }
    })?;
    // Start deadline checker
    deadline::start();
    // Run led task
    // Start temperature reader
    // Onboard RGB LED (ESP32-C3-DevKitC-02 pin gpio8)
    let pin = peripherals.pins.gpio8;
    let channel = peripherals.rmt.channel0;
    let led_sender = led::start(pin, channel)?;
    // {
    //     let pin = peripherals.pins.gpio4;
    //     let mut driver4 = PinDriver::input_output(pin)?;
    //     let pin = peripherals.pins.gpio5;
    //     let mut driver5 = PinDriver::input_output(pin)?;
    //     spawn(async move {
    //         loop {
    //             driver4.set_low().unwrap();
    //             info!("set_low");
    //             sleep(Duration::from_secs(2)).await;
    //             driver4.set_high().unwrap();
    //             info!("set_high");
    //             sleep(Duration::from_secs(2)).await;
    //         }
    //     });
    // }
    // {
    //     let led_sender = led_sender.clone();
    //     spawn(async move {
    //         loop {
    //             led_sender
    //                 .send(Ok(Duration::from_millis(100)))
    //                 .await
    //                 .unwrap();
    //             sleep(Duration::from_millis(500)).await;
    //             led_sender
    //                 .send(Err(Duration::from_millis(100)))
    //                 .await
    //                 .unwrap();
    //             sleep(Duration::from_millis(500)).await;
    //         }
    //     });
    // }
    // Run modbus server
    modbus::run(
        peripherals.pins.gpio4,
        peripherals.pins.gpio5,
        led_sender.clone(),
    )
    .await?;
    Ok(())
}

mod deadline;
mod led;
mod modbus;
mod relay;
mod wifi;
