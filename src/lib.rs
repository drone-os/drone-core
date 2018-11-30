//! Drone is a Hard Real-Time Operating System Framework for writing
//! embedded applications with [Rust].
//!
//! # Requirements
//!
//! * latest nightly [Rust];
//! * [xargo];
//! * [cargo-drone] host utility;
//!
//! Please also refer the installation notes of a corresponding Drone's
//! [platform implementation](#platforms).
//!
//! # Platforms
//!
//! * [STM32](https://github.com/drone-os/drone-stm32)
//!
//! # Demo Applications
//!
//! * [STM32 Nucleo L496ZG-P](https://github.com/drone-os/demo-core-nucleo)
//!
//! [Rust]: https://www.rust-lang.org/
//! [cargo-drone]: https://github.com/drone-os/cargo-drone
//! [xargo]: https://github.com/japaric/xargo

#![feature(alloc)]
#![feature(allocator_api)]
#![feature(associated_type_defaults)]
#![feature(const_fn)]
#![feature(core_intrinsics)]
#![feature(exhaustive_patterns)]
#![feature(generators)]
#![feature(generator_trait)]
#![feature(integer_atomics)]
#![feature(marker_trait_attr)]
#![feature(never_type)]
#![feature(optin_builtin_traits)]
#![feature(prelude_import)]
#![feature(raw_vec_internals)]
#![feature(slice_concat_ext)]
#![feature(slice_internals)]
#![feature(untagged_unions)]
#![warn(missing_docs)]
#![allow(clippy::precedence, clippy::inline_always)]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;
#[cfg(feature = "std")]
extern crate core;
extern crate drone_core_macros;
extern crate drone_ctypes;
extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate futures;

#[macro_use]
pub mod async;
pub mod bitfield;
pub mod ffi;
pub mod fib;
pub mod fs;
pub mod heap;
pub mod io;
pub mod mem;
pub mod prelude;
pub mod reg;
pub mod stack_adapter;
pub mod sv;
pub mod sync;
pub mod thr;

mod drv;

pub use drone_core_macros::{heap, thr, Bitfield};

#[prelude_import]
#[allow(unused_imports)]
use prelude::*;
