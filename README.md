[![crates.io](https://img.shields.io/crates/v/drone-core.svg)](https://crates.io/crates/drone-core)
![maintenance](https://img.shields.io/badge/maintenance-actively--developed-brightgreen.svg)

# Drone Core

<!-- cargo-rdme start -->

The core crate for Drone, an Embedded Operating System.

## Documentation

- [Drone Book](https://book.drone-os.com/)
- [API documentation](https://api.drone-os.com/drone-core/0.15/)

## Usage

Add the crate to your `Cargo.toml` dependencies:

```toml
[dependencies]
drone-core = { version = "0.15.0" }
```

Add or extend `host` feature as follows:

```toml
[features]
host = ["drone-core/host"]
```

<!-- cargo-rdme end -->

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
