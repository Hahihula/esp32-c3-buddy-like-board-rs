#![no_std]
#![no_main]

use embedded_graphics::{
    mono_font::{ascii::FONT_4X6, ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};

use core::fmt::Write as FmtWrite;
use esp_backtrace as _;
use esp_println::println;
use esp_wifi::{
    init,
    wifi::{
        utils::create_network_interface, AccessPointInfo, ClientConfiguration, Configuration,
        WifiError, WifiStaDevice,
    },
    wifi_interface::WifiStack,
    EspWifiInitFor,
};
use hal::{
    delay::Delay,
    gpio::Io,
    i2c,
    prelude::*,
    rng::Rng,
    time::{self},
    timer::timg::TimerGroup,
};
use smoltcp::iface::SocketStorage;

use sh1106::{prelude::*, Builder};
const SSID: &str = "SSID"; // env!("SSID");
const PASSWORD: &str = "PSSWD"; // env!("PASSWORD");

#[entry]
fn main() -> ! {
    esp_alloc::heap_allocator!(72 * 1024);

    let peripherals = hal::init(hal::Config::default());

    let timer = TimerGroup::new(peripherals.TIMG1).timer0;

    let delay = Delay::new();

    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    let sda = io.pins.gpio5;
    let scl = io.pins.gpio6;

    let i2c = i2c::I2c::new(peripherals.I2C0, sda, scl, 100u32.kHz());

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

    // with the 6x10 font, IP address is too long to fit on the screen
    let ip_text_style = MonoTextStyleBuilder::new()
        .font(&FONT_4X6)
        .text_color(BinaryColor::On)
        .build();

    // positions on the screen
    // the zero point on the screen is (28, 12)
    let starting_point = Point::new(28, 12);
    let ip_point = starting_point + Point::new(0, 36);

    display.clear();
    Text::with_baseline("WiFi example", starting_point, text_style, Baseline::Top)
        .draw(&mut display)
        .unwrap();

    display.flush().unwrap();

    let rng = Rng::new(peripherals.RNG);

    let init = init(EspWifiInitFor::Wifi, timer, rng, peripherals.RADIO_CLK)
        .map_err(|e| println!("Failed to initialize wifi {:?}", e))
        .unwrap();

    let wifi = peripherals.WIFI;
    let mut socket_set_entries: [SocketStorage; 5] = Default::default();
    let (iface, device, mut controller, sockets) =
        create_network_interface(&init, wifi, WifiStaDevice, &mut socket_set_entries).unwrap();

    let now = || time::now().duration_since_epoch().to_millis();

    let wifi_stack = WifiStack::new(iface, device, sockets, now);

    let client_config = Configuration::Client(ClientConfiguration {
        ssid: SSID.try_into().unwrap(),
        password: PASSWORD.try_into().unwrap(),
        ..Default::default()
    });
    let res = controller.set_configuration(&client_config);
    println!("wifi_set_configuration returned {:?}", res);

    controller.start().unwrap();
    println!("is wifi started: {:?}", controller.is_started());

    println!("Start Wifi Scan");
    let res: Result<(heapless::Vec<AccessPointInfo, 10>, usize), WifiError> = controller.scan_n();
    if let Ok((res, _count)) = res {
        for ap in res {
            println!("{:?}", ap);
        }
    }

    println!("{:?}", controller.get_capabilities());
    println!("wifi_connect {:?}", controller.connect());

    // wait to get connected
    println!("Wait to get connected");
    display.clear();
    Text::with_baseline(
        "WiFi example\nconnecting...",
        starting_point,
        text_style,
        Baseline::Top,
    )
    .draw(&mut display)
    .unwrap();
    match display.flush() {
        Ok(_) => (),
        Err(e) => println!("Error flushing display: {:?}", e),
    };

    loop {
        let res = controller.is_connected();
        match res {
            Ok(connected) => {
                if connected {
                    break;
                }
            }
            Err(err) => {
                println!("{:?}", err);
                loop {}
            }
        }
    }
    println!("{:?}", controller.is_connected());

    // wait for getting an ip address
    println!("Waiting for ip...");

    loop {
        wifi_stack.work();

        if wifi_stack.is_iface_up() {
            println!("got ip {:?}", wifi_stack.get_ip_info());

            let mut ip_addr: heapless::String<256> = heapless::String::new();
            let bytes = wifi_stack.get_ip_info().unwrap().ip.octets();
            match write!(
                ip_addr,
                "{}.{}.{}.{}",
                bytes[0], bytes[1], bytes[2], bytes[3]
            ) {
                Ok(_) => (),
                Err(e) => println!("Error writing ip: {:?}", e),
            }
            // .unwrap();
            display.clear();
            Text::with_baseline(
                "WiFi example\nConnected.\nIP:",
                starting_point,
                text_style,
                Baseline::Top,
            )
            .draw(&mut display)
            .unwrap();
            Text::new(&ip_addr, ip_point, ip_text_style)
                .draw(&mut display)
                .unwrap();
            match display.flush() {
                Ok(_) => (),
                Err(e) => println!("Error flushing display: {:?}", e),
            }
            break;
        }
    }

    println!("Start busy loop on main");

    let mut rx_buffer = [0u8; 1536];
    let mut tx_buffer = [0u8; 1536];
    let _socket = wifi_stack.get_socket(&mut rx_buffer, &mut tx_buffer);

    loop {}
}
