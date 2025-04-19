#![feature(impl_trait_in_assoc_type)]

use anyhow::Result;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{
        delay::FreeRtos, gpio::{PinDriver, Pull}, prelude::Peripherals, reset::restart
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
    time::{Duration, sleep},
};
use wifi::connect;

fn main() -> Result<()> {
    link_patches();
    EspLogger::initialize_default();
    // let _mounted_eventfs = MountedEventfs::mount(5)?;
    // info!("System initialized");
    // if let Err(error) = Builder::new_current_thread()
    //     .enable_all()
    //     .build()?
    //     .block_on(run())
    // {
    //     error!("{error:?}");
    // } else {
    //     info!("`main()` finished, restarting");
    // }
    // restart();
    let peripherals = Peripherals::take()?;
    let mut led = PinDriver::output(peripherals.pins.gpio4)?;

    loop {
        led.set_high()?;
        // we are sleeping here to make sure the watchdog isn't triggered
        FreeRtos::delay_ms(1000);

        led.set_low()?;
        FreeRtos::delay_ms(1000);
    }
}

async fn run() -> Result<()> {
    let event_loop = EspSystemEventLoop::take()?;
    let timer = EspTaskTimerService::new()?;
    let peripherals = Peripherals::take()?;
    let nvs = EspDefaultNvsPartition::take()?;
    // Initialize the network stack, this must be done before starting the server
    // let mut wifi = connect(peripherals.modem, event_loop.clone(), timer, Some(nvs)).await?;
    // let _subscription = event_loop.subscribe::<WifiEvent, _>(move |event| {
    //     info!("Got event: {event:?}");
    //     if let WifiEvent::StaDisconnected(_) = event {
    //         if let Err(error) = wifi.connect() {
    //             warn!("Wifi connect failed: {error}");
    //         }
    //     }
    // })?;
    // Start deadline checker
    // deadline::start();
    // Start temperature reader
    // let temperature_sender = relay::start(peripherals.pins.gpio2, peripherals.rmt.channel0)?;
    // Run modbus server
    // modbus::run(temperature_sender.clone()).await?;
    // Run led task
    let pin = peripherals.pins.gpio4;
    let mut driver = PinDriver::input_output(pin)?;
    // info!("driver: {driver:?}");
    // driver.set_pull(Pull::Floating)?;
    loop {
        driver.set_low()?;
        info!("set_low");
        sleep(Duration::from_secs(2)).await;
        driver.set_high()?;
        info!("set_high");
        sleep(Duration::from_secs(2)).await;
    }
}

// mod modbus;
// mod led;
// mod relay;
mod deadline;
mod wifi;
