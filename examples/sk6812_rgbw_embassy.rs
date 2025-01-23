//! Demonstrates generating pulse sequences with RMT
//!
//! Connect a logic analyzer to GPIO4 to see the generated pulses.
//!
//! The following wiring is assumed:
//! - generated pulses => GPIO4

//% CHIPS: esp32 esp32c3 esp32c6 esp32h2 esp32s2 esp32s3
//% FEATURES: embassy esp-hal/unstable

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{
    rmt::{PulseCode, Rmt, TxChannelAsync, TxChannelConfig, TxChannelCreatorAsync},
    time::RateExtU32,
    timer::timg::TimerGroup,
};
use esp_println::println;

const T0H: u16 = 40; // 0.4us
const T0L: u16 = 85; // 0.85us
const T1H: u16 = 80; // 0.8us
const T1L: u16 = 45; // 0.45us

fn create_led_bits(r: u8, g: u8, b: u8, w: u8) -> [u32; 33] {
  let mut data = [PulseCode::empty(); 33];
  let bytes = [g, r, b, w];
  
  let mut idx = 0;
  for byte in bytes {
      for bit in (0..8).rev() {
          data[idx] = if (byte & (1 << bit)) != 0 {
              PulseCode::new(true, T1H, false, T1L)
          } else {
              PulseCode::new(true, T0H, false, T0L)
          };
          idx += 1;
      }
  }
  data[32] = PulseCode::new(false, 800, false, 0);
  data
}

#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    let freq = 80.MHz();
    let rmt = Rmt::new(peripherals.RMT, freq).unwrap().into_async();

    let mut channel = rmt
        .channel0
        .configure(
            peripherals.GPIO4,
            TxChannelConfig {
                clk_divider: 1,
                ..TxChannelConfig::default()
            },
        )
        .unwrap();

    
    let led_colors = [
        (5, 0, 0, 0),    // Red
        (0, 5, 0, 0),    // Green
        (0, 0, 5, 0),    // Blue
        (0, 0, 0, 5),    // White
    ];

    loop {
      println!("Settings LED colors:");
        for &(r, g, b, w) in led_colors.iter() {
            let data = create_led_bits(r, g, b, w);
            channel.transmit(&data).await.unwrap();
        }
        Timer::after(Duration::from_millis(100)).await;
    }
}