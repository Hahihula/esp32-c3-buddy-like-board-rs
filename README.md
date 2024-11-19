# esp32-c3-buddy-like board

## up and running

`cargo build --release`

`cargo espflash flash --release`

`cargo espflash monitor`

## Examples

- [blink](examples/blink.rs)
  `cargo espflash flash --release --example blink`

- [snow](examples/snow.rs)
  `cargo espflash flash --release --example snow`
