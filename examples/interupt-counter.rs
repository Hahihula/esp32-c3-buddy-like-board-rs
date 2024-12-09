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
    gpio::{Event, Input, Io, Pull},
    i2c,
    prelude::*,
    time::{self},
};

use sh1106::{prelude::*, Builder};

use core::cell::RefCell;
use critical_section::Mutex;

static BUTTON: Mutex<RefCell<Option<Input>>> = Mutex::new(RefCell::new(None));

static COUNTER: Mutex<RefCell<u32>> = Mutex::new(RefCell::new(0));

#[entry]
fn main() -> ! {
    let peripherals = hal::init(hal::Config::default());

    let delay = Delay::new();

    let mut io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    io.set_interrupt_handler(handler);

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
    let mut button = Input::new(button_pin, Pull::Up);

    critical_section::with(|cs| {
        COUNTER.borrow_ref_mut(cs);
        button.listen(Event::FallingEdge);
        BUTTON.borrow_ref_mut(cs).replace(button)
    });

    let mut last_counter_change = time::now().duration_since_epoch().to_millis();

    loop {
        display.clear();
        Text::with_baseline("Counter:", starting_point, text_style, Baseline::Top)
            .draw(&mut display)
            .unwrap();

        // Get current counter value from the mutex
        let current_counter = critical_section::with(|cs| *COUNTER.borrow_ref(cs));

        // Convert counter to string and display it
        let mut counter_string: heapless::String<256> = heapless::String::new();
        match write!(counter_string, "{}", current_counter) {
            Ok(_) => (),
            Err(e) => println!("Error writing counter: {:?}", e),
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

#[handler]
#[ram]
fn handler() {
    esp_println::println!("GPIO Interrupt");

    if critical_section::with(|cs| {
        BUTTON
            .borrow_ref_mut(cs)
            .as_mut()
            .unwrap()
            .is_interrupt_set()
    }) {
        esp_println::println!("Button was the source of the interrupt");
        critical_section::with(|cs| {
            let mut counter = COUNTER.borrow_ref_mut(cs);
            *counter += 1;
            println!("Counter incremented to: {}", *counter);
        });
    } else {
        esp_println::println!("Button was not the source of the interrupt");
    }

    critical_section::with(|cs| {
        BUTTON
            .borrow_ref_mut(cs)
            .as_mut()
            .unwrap()
            .clear_interrupt()
    });
}
