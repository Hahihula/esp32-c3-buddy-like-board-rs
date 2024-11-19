#![no_std]
#![no_main]

use embedded_graphics::{
  mono_font::{ascii::FONT_4X6, MonoTextStyleBuilder},
  pixelcolor::BinaryColor,
  prelude::*,
  text::{Baseline, Text},
};

use esp_backtrace as _;
use hal::{
    delay::Delay,
    gpio::Io,
    i2c,
    rng::Rng,
    prelude::*,
};
use esp_println::println;

use sh1106::{prelude::*, Builder};

#[entry]
fn main() -> ! {
    let peripherals = hal::init(hal::Config::default());

    let delay = Delay::new();

    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    let sda = io.pins.gpio5;
    let scl = io.pins.gpio6;

    let i2c = i2c::I2c::new(peripherals.I2C0, sda, scl, 400u32.kHz());

    let mut display: GraphicsMode<_> = Builder::new().connect_i2c(i2c).into();
    match display.init() {
        Ok(_) => (),
        Err(e) => println!("Error initializing display: {:?}", e),
    }
    display.flush().unwrap();

    let snow_style = MonoTextStyleBuilder::new()
        .font(&FONT_4X6)
        .text_color(BinaryColor::On)
        .build();

    // Instantiate the hardware RNG:
    let mut rng = Rng::new(peripherals.RNG);

    // Number of snowflakes and their positions
    let mut snowflakes = [(0, 0); 10]; // Adjust the number of snowflakes here
    
    loop {
      display.clear();

      // Update snowflake positions
      for snowflake in snowflakes.iter_mut() {
          // Randomly generate new snowflakes at the top
          if rng.random() % 20 == 0 {
              snowflake.0 = (rng.random() % 128) as i32;
              snowflake.1 = 0;
          } else {
              // Adjust for 45-degree tilt
              snowflake.1 += 1;
              snowflake.0 -= 1; // Adjust this value if needed for correct tilt compensation

              // Check bounds and reset if needed
              if snowflake.1 > 48 {
                  snowflake.1 = 0;
                  snowflake.0 = (rng.random() % 128) as i32; // Reset x position too
              }
              if snowflake.0 < 0 {
                  snowflake.0 = 128; // Wrap around if it goes off the left edge
              }
          }

          // Draw snowflake
          Text::with_baseline(
              "*",
              Point::new(snowflake.0, snowflake.1),
              snow_style,
              Baseline::Top,
          )
          .draw(&mut display)
          .unwrap();
      }

      display.flush().unwrap();
      delay.delay_millis(100u32); // Adjust for snowflake fall speed
  }
}
