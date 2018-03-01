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
#![feature(conservative_impl_trait)]
#![feature(const_atomic_bool_new)]
#![feature(const_atomic_ptr_new)]
#![feature(const_atomic_usize_new)]
#![feature(const_fn)]
#![feature(const_max_value)]
#![feature(const_min_value)]
#![feature(const_ptr_null_mut)]
#![feature(const_size_of)]
#![feature(const_unsafe_cell_new)]
#![feature(fused)]
#![feature(generators)]
#![feature(generator_trait)]
#![feature(integer_atomics)]
#![feature(iterator_for_each)]
#![feature(macro_reexport)]
#![feature(never_type)]
#![feature(nonzero)]
#![feature(optin_builtin_traits)]
#![feature(pointer_methods)]
#![feature(prelude_import)]
#![feature(proc_macro)]
#![feature(proc_macro)]
#![feature(slice_concat_ext)]
#![feature(slice_get_slice)]
#![feature(unreachable)]
#![feature(untagged_unions)]
#![warn(missing_docs)]
#![doc(html_root_url = "https://docs.rs/drone-core/0.8.0")]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![cfg_attr(feature = "clippy", allow(precedence, inline_always))]

extern crate alloc;
#[macro_reexport(Bitfield)]
extern crate drone_core_macros;
extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate futures;

#[cfg(feature = "std")]
extern crate core;

pub mod async;
pub mod bitfield;
pub mod drv;
pub mod fib;
pub mod heap;
pub mod io;
pub mod mem;
pub mod prelude;
pub mod reg;
pub mod sync;
pub mod thr;

pub use drone_core_macros::{heap, thr};

#[prelude_import]
#[allow(unused_imports)]
use prelude::*;
