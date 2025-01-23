#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_println::println;
use hal::{
    delay::Delay,
    gpio::{Io, Level, Output},
    prelude::*,
    rmt::{PulseCode, Rmt, TxChannel, TxChannelConfig, TxChannelCreator},
};

#[entry]
fn main() -> ! {
    let peripherals = hal::init(hal::Config::default());

    // Set GPIO8 as an output, and set its state high initially.
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);
    let mut led = Output::new(io.pins.gpio8, Level::High);

    let delay = Delay::new();

    let freq = 80.MHz(); // 80 MHz is the maximum frequency for RMT on ESP32-C3 ( for esp32h2 it is 32 MHz)

    let rmt = Rmt::new(peripherals.RMT, freq).unwrap();
    let channel = rmt
        .channel0
        .configure(io.pins.gpio3, TxChannelConfig::default())
        .unwrap();

    let mut data = [PulseCode::from(350); 20];
    data[data.len() - 2] = PulseCode::from(250);
    data[data.len() - 1] = PulseCode::from(350);

    println!("transmit");
    channel.transmit(&data);
    println!("transmitted\n");

    loop {
        led.toggle();
        delay.delay_millis(500);
        led.toggle();
        println!("Blink!");
        delay.delay(1.secs());
    }
}
