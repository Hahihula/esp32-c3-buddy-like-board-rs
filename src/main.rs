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
    gpio::{Event, Input, Io, Pull},
    prelude::*,
    rng::Rng,
    time::{self, Duration},
    timer::timg::TimerGroup,
};

use core::cell::RefCell;
use critical_section::Mutex;

static BUTTON: Mutex<RefCell<Option<Input>>> = Mutex::new(RefCell::new(None));

static COUNTER: Mutex<RefCell<u32>> = Mutex::new(RefCell::new(0));

// Circular buffer to store counts for each second

#[entry]
fn main() -> ! {
    esp_alloc::heap_allocator!(72 * 1024);
    let peripherals = hal::init({
        let mut config = hal::Config::default();
        config.cpu_clock = CpuClock::max();
        config
    });

    let mut io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    io.set_interrupt_handler(handler);

    let button_pin = io.pins.gpio3;
    let mut button = Input::new(button_pin, Pull::None);

    critical_section::with(|cs| {
        COUNTER.borrow_ref_mut(cs);
        button.listen(Event::FallingEdge);
        BUTTON.borrow_ref_mut(cs).replace(button)
    });

    let timg0 = TimerGroup::new(peripherals.TIMG0);

    let rng = Rng::new(peripherals.RNG);

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

    let mut next_send_time = time::now() + Duration::secs(1);

    let mut second_counts: [u32; 60] = [0; 60];
    let mut current_index: usize = 0;

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
            next_send_time = time::now() + Duration::secs(1);
            println!("Send");
            let message: [u8; 1] = 42_u8.to_be_bytes();

            // Get current counter value from the mutex
            let current_counter = critical_section::with(|cs| {
                let value = *COUNTER.borrow_ref_mut(cs);
                *COUNTER.borrow_ref_mut(cs) = 0;
                value
            });
            second_counts[current_index] = current_counter;
            current_index = (current_index + 1) % 60;

            let total = second_counts.iter().sum::<u32>();

            // let random_number: u32 = rng.random() % 128;
            let mut counter_string: heapless::String<4> = heapless::String::new();
            match write!(counter_string, "{}", total) {
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
