#![no_std]
#![no_main]
use core::fmt::Write as FmtWrite;
use esp_backtrace as _;
use esp_println::println;
use esp_wifi::{
    esp_now::{PeerInfo, BROADCAST_ADDRESS},
    init, EspWifiInitFor,
};
use hal::{
    prelude::*,
    rng::Rng,
    time::{self, Duration},
    timer::timg::TimerGroup,
};

#[entry]
fn main() -> ! {
    esp_alloc::heap_allocator!(72 * 1024);
    let peripherals = hal::init({
        let mut config = hal::Config::default();
        config.cpu_clock = CpuClock::max();
        config
    });

    let timg0 = TimerGroup::new(peripherals.TIMG0);

    let mut rng = Rng::new(peripherals.RNG);

    let init = init(
        EspWifiInitFor::Wifi,
        timg0.timer0,
        rng,
        peripherals.RADIO_CLK,
    )
    .unwrap();

    let wifi = peripherals.WIFI;
    let mut esp_now = esp_wifi::esp_now::EspNow::new(&init, wifi).unwrap();

    println!("esp-now version {}", esp_now.get_version().unwrap());

    let mut next_send_time = time::now() + Duration::secs(5);

    loop {
        let r = esp_now.receive();
        if let Some(r) = r {
            println!("Received {:?}", r);

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
        }

        if time::now() >= next_send_time {
            next_send_time = time::now() + Duration::secs(5);
            println!("Send");
            let message: [u8; 1] = 42_u8.to_be_bytes();
            let random_number: u32 = rng.random() % 128;
            let mut counter_string: heapless::String<4> = heapless::String::new();
            match write!(counter_string, "{}", random_number) {
                Ok(_) => (),
                Err(e) => println!("Error writing: {:?}", e),
            }
            let status = esp_now
                .send(&BROADCAST_ADDRESS, counter_string.as_bytes())
                .unwrap()
                .wait();
            println!("Send broadcast status: {:?}", status)
        }
    }
}
