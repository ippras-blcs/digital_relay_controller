// use smart_leds::{
//     SmartLedsWrite, brightness, gamma,
//     hsv::{Hsv, hsv2rgb},
// };
use anyhow::{Error, Result};
use esp_idf_svc::hal::{
    gpio::OutputPin,
    peripheral::Peripheral,
    rmt::{FixedLengthSignal, PinState, Pulse, RmtChannel, TxRmtDriver, config::TransmitConfig},
};
use log::{info, trace};
use tokio::{
    spawn,
    sync::mpsc::{self, Sender},
    time::{Duration, sleep},
};

const DELAY: Duration = Duration::from_millis(3000);
const SLEEP: Duration = Duration::from_millis(10);

pub(crate) type Request = Result<Duration, Duration>;

pub(crate) fn start(
    pin: impl Peripheral<P = impl OutputPin> + 'static,
    channel: impl Peripheral<P = impl RmtChannel> + 'static,
) -> Result<Sender<Request>> {
    let config = TransmitConfig::new().clock_divider(1);
    let mut driver = TxRmtDriver::new(channel, pin, &config)?;
    info!("LED driver initialized");
    let (sender, mut receiver) = mpsc::channel::<Request>(9);
    info!("Spawn LED receiver");
    spawn(async move {
        while let Some(request) = receiver.recv().await {
            trace!("Read LED {request:?}");
            match request {
                Ok(duration) => {
                    neopixel(&mut driver, Rgb::new(0, 255, 0))?;
                    sleep(duration).await;
                }
                Err(duration) => {
                    neopixel(&mut driver, Rgb::new(255, 0, 0))?;
                    sleep(duration).await;
                }
            }
            neopixel(&mut driver, Rgb::new(0, 0, 0))?;
        }
        Result::<_, Error>::Ok(())
    });
    Ok(sender)
}

fn neopixel(driver: &mut TxRmtDriver, rgb: Rgb) -> Result<()> {
    let color: u32 = rgb.into();
    let ticks_hz = driver.counter_clock()?;
    let (t0h, t0l, t1h, t1l) = (
        Pulse::new_with_duration(ticks_hz, PinState::High, &Duration::from_nanos(350))?,
        Pulse::new_with_duration(ticks_hz, PinState::Low, &Duration::from_nanos(800))?,
        Pulse::new_with_duration(ticks_hz, PinState::High, &Duration::from_nanos(700))?,
        Pulse::new_with_duration(ticks_hz, PinState::Low, &Duration::from_nanos(600))?,
    );
    let mut signal = FixedLengthSignal::<24>::new();
    for i in (0..24).rev() {
        let p = 2_u32.pow(i);
        let bit: bool = p & color != 0;
        let (high_pulse, low_pulse) = if bit { (t1h, t1l) } else { (t0h, t0l) };
        signal.set(23 - i as usize, &(high_pulse, low_pulse))?;
    }
    driver.start_blocking(&signal)?;
    Ok(())
}

struct Rgb {
    r: u8,
    g: u8,
    b: u8,
}

impl Rgb {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    // /// Converts hue, saturation, value to RGB
    // pub fn from_hsv(h: u32, s: u32, v: u32) -> Result<Self> {
    //     if h > 360 || s > 100 || v > 100 {
    //         bail!("The given HSV values are not in valid range");
    //     }
    //     let s = s as f64 / 100.0;
    //     let v = v as f64 / 100.0;
    //     let c = s * v;
    //     let x = c * (1.0 - (((h as f64 / 60.0) % 2.0) - 1.0).abs());
    //     let m = v - c;
    //     let (r, g, b) = match h {
    //         0..=59 => (c, x, 0.0),
    //         60..=119 => (x, c, 0.0),
    //         120..=179 => (0.0, c, x),
    //         180..=239 => (0.0, x, c),
    //         240..=299 => (x, 0.0, c),
    //         _ => (c, 0.0, x),
    //     };
    //     Ok(Self {
    //         r: ((r + m) * 255.0) as u8,
    //         g: ((g + m) * 255.0) as u8,
    //         b: ((b + m) * 255.0) as u8,
    //     })
    // }
}

impl From<Rgb> for u32 {
    /// Convert RGB to u32 color value
    ///
    /// e.g. rgb: (1,2,4)
    /// G        R        B
    /// 7      0 7      0 7      0
    /// 00000010 00000001 00000100
    fn from(rgb: Rgb) -> Self {
        ((rgb.g as u32) << 16) | ((rgb.r as u32) << 8) | rgb.b as u32
    }
}

// const BUFFER_LENGTH: usize = 1;
// const BUFFER_SIZE: usize = BUFFER_LENGTH * 24 + 1;

// pub struct Led<T: TxChannel = Channel<Blocking, 0>> {
//     adapter: SmartLedsAdapter<T, BUFFER_SIZE>,
// }

// impl<'a> Led {
//     pub fn new(
//         pin: impl Peripheral<P = impl OutputPin> + 'a,
//         rmt: impl Peripheral<P = RMT> + 'a,
//         // channel: impl Peripheral<P = impl TxChannel> + 'a,
//     ) -> Self {
//         let rmt = Rmt::new(rmt, Rate::from_mhz(80)).unwrap();
//         let adapter = SmartLedsAdapter::new(rmt.channel0, pin, smartLedBuffer!(1));
//         // let config = TransmitConfig::new().clock_divider(2);
//         // let tx = RmtDriver::new(channel, pin, &config)?;
//         Self { adapter }
//     }
// }
