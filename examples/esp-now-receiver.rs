#![no_std]
#![no_main]

use core::fmt::Write as FmtWrite;
use core::str;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use esp_backtrace as _;
use esp_println::println;
use esp_wifi::{
    esp_now::{PeerInfo, BROADCAST_ADDRESS},
    init, EspWifiInitFor,
};
use hal::{gpio::Io, i2c, prelude::*, rng::Rng, timer::timg::TimerGroup};
use sh1106::{prelude::*, Builder};

fn bytes_to_ascii_string(bytes: &[u8; 256]) -> Result<&str, str::Utf8Error> {
    // Find the actual length by looking for null terminator or non-ASCII bytes
    let len = bytes
        .iter()
        .position(|&b| b == 0 || b > 127)
        .unwrap_or(bytes.len());

    // Take the slice up to the determined length
    let valid_bytes = &bytes[..len];

    // Convert to str, this is safe because we've verified ASCII-only content
    str::from_utf8(valid_bytes)
}

#[entry]
fn main() -> ! {
    esp_alloc::heap_allocator!(72 * 1024);
    let peripherals = hal::init({
        let mut config = hal::Config::default();
        config.cpu_clock = CpuClock::max();
        config
    });

    let timg0 = TimerGroup::new(peripherals.TIMG0);

    let init = init(
        EspWifiInitFor::Wifi,
        timg0.timer0,
        Rng::new(peripherals.RNG),
        peripherals.RADIO_CLK,
    )
    .unwrap();

    let wifi = peripherals.WIFI;
    let mut esp_now = esp_wifi::esp_now::EspNow::new(&init, wifi).unwrap();

    println!("esp-now version {}", esp_now.get_version().unwrap());

    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    let sda = io.pins.gpio5;
    let scl = io.pins.gpio6;

    let i2c = i2c::I2c::new(peripherals.I2C0, sda, scl, 400u32.kHz());

    // // positions on the screen
    // // the zero point on the screen is (28, 12)
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

    display.clear();
    Text::with_baseline(
        "ESP-NOW receiver:",
        starting_point,
        text_style,
        Baseline::Top,
    )
    .draw(&mut display)
    .unwrap();
    display.flush().unwrap();

    loop {
        let r = esp_now.receive();
        if let Some(r) = r {
            let message = match bytes_to_ascii_string(&r.data) {
                Ok(s) => s,
                Err(e) => {
                    println!("Error decoding message: {:?}", e);
                    continue;
                }
            };
            println!("Received message: {}", message);

            if r.info.dst_address == BROADCAST_ADDRESS {
                if !esp_now.peer_exists(&r.info.src_address) {
                    esp_now
                        .add_peer(PeerInfo {
                            peer_address: r.info.src_address,
                            lmk: None,
                            channel: None,
                            encrypt: false,
                        })
                        .unwrap();
                }
                let status = esp_now
                    .send(&r.info.src_address, b"Hello Peer")
                    .unwrap()
                    .wait();
                println!("Send hello to peer status: {:?}", status);
            }
            display.clear();
            Text::with_baseline("Received:", starting_point, text_style, Baseline::Top)
                .draw(&mut display)
                .unwrap();
            let mut counter_string: heapless::String<256> = heapless::String::new();
            match write!(counter_string, "{}", message) {
                Ok(_) => (),
                Err(e) => println!("Error writing ip: {:?}", e),
            }
            Text::with_baseline(
                &counter_string,
                Point::new(starting_point.x, starting_point.y + 10),
                text_style,
                Baseline::Top,
            )
            .draw(&mut display)
            .unwrap();

            display.flush().unwrap();
        }
    }
}
