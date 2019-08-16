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

#![feature(alloc_prelude)]
#![feature(allocator_api)]
#![feature(associated_type_defaults)]
#![feature(const_fn)]
#![feature(const_raw_ptr_deref)]
#![feature(core_intrinsics)]
#![feature(exhaustive_patterns)]
#![feature(generator_trait)]
#![feature(generators)]
#![feature(marker_trait_attr)]
#![feature(maybe_uninit_extra)]
#![feature(maybe_uninit_ref)]
#![feature(never_type)]
#![feature(optin_builtin_traits)]
#![feature(prelude_import)]
#![feature(raw_vec_internals)]
#![feature(result_map_or_else)]
#![feature(slice_internals)]
#![feature(todo_macro)]
#![feature(untagged_unions)]
#![deny(elided_lifetimes_in_paths)]
#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::enum_glob_use,
    clippy::module_name_repetitions,
    clippy::precedence,
    clippy::type_repetition_in_bounds,
    clippy::use_self
)]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod bitfield;
pub mod ffi;
pub mod fib;
pub mod future;
pub mod heap;
pub mod inventory;
pub mod io;
pub mod mem;
pub mod periph;
pub mod prelude;
pub mod reg;
pub mod stack_loop;
pub mod sv;
pub mod sync;
pub mod thr;
pub mod token;

/// Defines dynamic memory structures.
///
/// See [the module level documentation][heap] for details.
pub use drone_core_macros::heap;

/// Defines peripherals.
///
/// See [the module level documentation][periph] for details.
pub use drone_core_macros::periph;

/// Defines a memory-mapped register.
///
/// See [the module level documentation][reg] for details.
pub use drone_core_macros::reg;

/// Defines threads.
///
/// See [the module level documentation][thr] for details.
pub use drone_core_macros::thr;

#[prelude_import]
#[allow(unused_imports)]
use crate::prelude::*;
