#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_println::println;
use hal::ledc::channel::config::PinConfig;
use hal::{
    delay::Delay,
    gpio::{Io, Level, Output},
    prelude::*,
    timer::Timer,
};

#[derive(Debug, Clone, Copy)]
struct RgbwColor {
    r: u8,
    g: u8,
    b: u8,
    w: u8,
}

struct Sk6812Strip<'a> {
    data_pin: Output<'a>,
    delay: Delay,
    buffer: [RgbwColor; NUM_LEDS],
}

impl<'a> Sk6812Strip<'a> {
    fn new(data_pin: Output<'a>) -> Self {
        Self {
            data_pin,
            delay: Delay::new(),
            buffer: [RgbwColor {
                r: 0,
                g: 0,
                b: 0,
                w: 0,
            }; NUM_LEDS],
        }
    }

    fn set_pixel(&mut self, index: usize, color: RgbwColor) {
        if index < NUM_LEDS {
            self.buffer[index] = color;
        }
    }

    fn show(&mut self) {
        critical_section::with(|_| {
            for i in 0..NUM_LEDS {
                let pixel = self.buffer[i]; // Copy the pixel data
                self.send_byte(pixel.g);
                self.send_byte(pixel.r);
                self.send_byte(pixel.b);
                self.send_byte(pixel.w);
            }
        });

        // Reset signal (>50µs low)
        self.data_pin.set_low();
        self.delay.delay_micros(60);
    }

    fn send_byte(&mut self, mut byte: u8) {
        for _ in 0..8 {
            if (byte & 0x80) != 0 {
                // One bit (800ns high, 450ns low)
                self.data_pin.set_high();
                self.delay.delay_nanos(800);
                self.data_pin.set_low();
                self.delay.delay_nanos(450);
            } else {
                // Zero bit (400ns high, 850ns low)
                self.data_pin.set_high();
                self.delay.delay_nanos(400);
                self.data_pin.set_low();
                self.delay.delay_nanos(850);
            }
            byte <<= 1;
        }
    }
}

const NUM_LEDS: usize = 5;
#[entry]
fn main() -> ! {
    let peripherals = hal::init(hal::Config::default());

    // Set GPIO8 as an output, and set its state high initially.
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);
    let mut led = Output::new(io.pins.gpio8, Level::High);

    let delay = Delay::new();

    // Initialize the LED strip on GPIO3
    let led_pin = Output::new(io.pins.gpio3, Level::Low);
    let mut strip = Sk6812Strip::new(led_pin);

    // Example colors
    let red = RgbwColor {
        r: 15,
        g: 0,
        b: 0,
        w: 0,
    };
    let green = RgbwColor {
        r: 0,
        g: 15,
        b: 0,
        w: 0,
    };
    let blue = RgbwColor {
        r: 0,
        g: 0,
        b: 15,
        w: 0,
    };
    let white = RgbwColor {
        r: 0,
        g: 0,
        b: 0,
        w: 15,
    };
    let purple = RgbwColor {
        r: 15,
        g: 0,
        b: 15,
        w: 0,
    };

    loop {
        led.toggle();
        delay.delay_millis(500);
        led.toggle();
        println!("Blink!");
        strip.set_pixel(0, red);
        strip.set_pixel(1, green);
        strip.set_pixel(2, blue);
        strip.set_pixel(3, white);
        strip.set_pixel(4, purple);
        println!("Colors set!");
        strip.show();
        delay.delay(1.secs());
        for i in 0..NUM_LEDS {
            strip.set_pixel(
                i,
                RgbwColor {
                    r: 0,
                    g: 0,
                    b: 0,
                    w: 0,
                },
            );
        }
        strip.show();
        println!("Colors reset!");
        delay.delay(1.secs());
    }
}
