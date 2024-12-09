# esp32-c3-buddy-like board

<img src="https://github.com/user-attachments/assets/1ce4e032-0fc8-434c-8a3f-b22e013efab5" height="128">

## up and running

`cargo build --release`

`cargo espflash flash --release`

`cargo espflash monitor`

## Examples

- [blink](examples/blink.rs)
  `cargo espflash flash --release --example blink`

- [snow](examples/snow.rs)
  `cargo espflash flash --release --example snow`

- [wifi](examples/wifi.rs)
  `cargo espflash flash --release --example wifi`

- [counter](examples/counter.rs)
  `cargo espflash flash --release --example counter`

- [interupt counter](examples/interupt-counter.rs)
  `cargo espflash flash --release --example interupt-counter`

- [esp-now-no-display](examples/esp-now-no-display.rs)
  `cargo espflash flash --release --example esp-now-no-display`

- [esp-now-receiver](examples/esp-now-receiver.rs)
  `cargo espflash flash --release --example esp-now-receiver`
