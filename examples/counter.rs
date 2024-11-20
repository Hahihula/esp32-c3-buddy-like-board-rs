#![no_std]
#![no_main]

use core::fmt::Write as FmtWrite;
use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use esp_backtrace as _;
use esp_println::println;
use hal::{
    delay::Delay,
    gpio::{Input, Io, Pull},
    i2c,
    prelude::*,
    time::{self},
};

use sh1106::{prelude::*, Builder};

#[entry]
fn main() -> ! {
    let peripherals = hal::init(hal::Config::default());

    let delay = Delay::new();

    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    let sda = io.pins.gpio5;
    let scl = io.pins.gpio6;

    let i2c = i2c::I2c::new(peripherals.I2C0, sda, scl, 400u32.kHz());

    // positions on the screen
    // the zero point on the screen is (28, 12)
    let starting_point = Point::new(28, 12);

    let mut display: GraphicsMode<_> = Builder::new().connect_i2c(i2c).into();
    match display.init() {
        Ok(_) => (),
        Err(e) => println!("Error initializing display: {:?}", e),
    }
    display.flush().unwrap();

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    let number_style = MonoTextStyleBuilder::new()
        .font(&FONT_10X20)
        .text_color(BinaryColor::On)
        .build();

    let button_pin = io.pins.gpio9.degrade();
    let button_pin = Input::new(button_pin, Pull::Up);

    let mut counter = 0;

    let mut last_counter_change = time::now().duration_since_epoch().to_millis();

    loop {
        display.clear();
        Text::with_baseline("Counter:", starting_point, text_style, Baseline::Top)
            .draw(&mut display)
            .unwrap();
        if button_pin.is_low() {
            let now = time::now().duration_since_epoch().to_millis();
            // Only increment once every 100ms to avoid more than one increment per button press
            if now - last_counter_change > 100 {
                last_counter_change = now;
                println!("Button pressed! Counter: {}", counter);
                counter += 1;
            }
        };
        let mut counter_string: heapless::String<256> = heapless::String::new();
        match write!(counter_string, "{}", counter) {
            Ok(_) => (),
            Err(e) => println!("Error writing ip: {:?}", e),
        }
        Text::with_baseline(
            &counter_string,
            Point::new(starting_point.x + 30, starting_point.y + 20),
            number_style,
            Baseline::Top,
        )
        .draw(&mut display)
        .unwrap();

        display.flush().unwrap();
        delay.delay_millis(30u32);
    }
}
