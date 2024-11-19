//! Blinks an LED
//!
//! The following wiring is assumed:
//! - LED => GPIO8

#![no_std]
#![no_main]

use esp_backtrace as _;
use hal::{
    delay::Delay,
    gpio::{Io, Level, Output},
    prelude::*,
};
use esp_println::println;

#[entry]
fn main() -> ! {
    let peripherals = hal::init(hal::Config::default());

    // Set GPIO8 as an output, and set its state high initially.
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);
    let mut led = Output::new(io.pins.gpio8, Level::High);

    let delay = Delay::new();

    loop {
        led.toggle();
        delay.delay_millis(500);
        led.toggle();
        println!("Blink!");
        delay.delay(1.secs());
    }
}
