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
//! * [ARM Cortex-M](https://github.com/valff/drone-cortex-m)
//!
//! # Example Applications
//!
//! * [STM32 Nucleo L496ZG-P](https://github.com/valff/blink-nucleo)
//!
//! [Rust]: https://www.rust-lang.org/
//! [cargo-drone]: https://github.com/valff/cargo-drone
//! [xargo]: https://github.com/japaric/xargo
#![feature(alloc)]
#![feature(allocator_api)]
#![feature(const_atomic_ptr_new)]
#![feature(const_atomic_usize_new)]
#![feature(const_fn)]
#![feature(const_ptr_null_mut)]
#![feature(decl_macro)]
#![feature(fused)]
#![feature(generators)]
#![feature(generator_trait)]
#![feature(iterator_for_each)]
#![feature(optin_builtin_traits)]
#![feature(pointer_methods)]
#![feature(proc_macro)]
#![feature(slice_get_slice)]
#![feature(untagged_unions)]
#![warn(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![cfg_attr(feature = "clippy", allow(precedence, doc_markdown))]

extern crate alloc;
extern crate drone_macros;

pub mod prelude;
pub mod reg;
pub mod heap;
pub mod mem;
pub mod routine;
pub mod collections;

pub use heap::heap;
pub use reg::reg;

#[cfg(feature = "std")]
use std as core;
