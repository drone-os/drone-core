Drone
=====

*Drone* is a [Real-Time Operating System][rtos] framework written in
[Rust][rust].

**Warning**: It is at the early experimental stage.

## Requirements

Instructions will be given for Debian-based Linux systems.

Install the following packages:

```sh
$ sudo apt-get install build-essential cmake libusb-1.0-0 libusb-1.0-0-dev \
  pandoc gcc-arm-none-eabi gdb-arm-none-eabi qemu-system-arm qemu-user
```

Copy [udev rules][rules.d] for ST-Link programmer to the `/etc/udev/rules.d/`,
and run the following commands:

```sh
$ sudo udevadm control --reload-rules
$ sudo udevadm trigger
```

[OpenOCD][openocd] is required.  It is recommended to install it from the
source, because repository package is outdated and doesn't contain configuration
for newer chips and boards.

Install [Rust][rust] if it is not installed (nightly channel is required), and
install [the cargo subcommand for Drone][cargo-drone] and [Xargo][xargo]:

```sh
$ rustup component add rust-src
$ cargo intall xargo
$ cargo intall cargo-drone
```

## Examples

* [STM32F1](https://github.com/valff/blink-stm32f1)
* [STM32 Nucleo L496ZG-P](https://github.com/valff/blink-nucleo)

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

[rust]: https://www.rust-lang.org/
[rtos]: https://en.wikipedia.org/wiki/Real-time_operating_system
[openocd]: http://openocd.org/
[rules.d]: https://github.com/texane/stlink/tree/master/etc/udev/rules.d
[cargo-drone]: https://github.com/valff/cargo-drone
[xargo]: https://github.com/japaric/xargo
