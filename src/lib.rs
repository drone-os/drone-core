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
//! * [ARM Cortex-M](https://github.com/drone-os/drone-cortex-m)
//!
//! # Example Applications
//!
//! * [STM32 Nucleo L496ZG-P](https://github.com/drone-os/blink-nucleo)
//!
//! [Rust]: https://www.rust-lang.org/
//! [cargo-drone]: https://github.com/drone-os/cargo-drone
//! [xargo]: https://github.com/japaric/xargo
#![feature(alloc)]
#![feature(allocator_api)]
#![feature(const_atomic_bool_new)]
#![feature(const_atomic_ptr_new)]
#![feature(const_atomic_usize_new)]
#![feature(const_fn)]
#![feature(const_max_value)]
#![feature(const_min_value)]
#![feature(const_ptr_null_mut)]
#![feature(const_unsafe_cell_new)]
#![feature(core_intrinsics)]
#![feature(decl_macro)]
#![feature(fused)]
#![feature(generators)]
#![feature(generator_trait)]
#![feature(integer_atomics)]
#![feature(iterator_for_each)]
#![feature(nonzero)]
#![feature(optin_builtin_traits)]
#![feature(pointer_methods)]
#![feature(prelude_import)]
#![feature(proc_macro)]
#![feature(slice_concat_ext)]
#![feature(slice_get_slice)]
#![feature(untagged_unions)]
#![warn(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![cfg_attr(feature = "clippy", allow(precedence, doc_markdown, inline_always))]

extern crate alloc;
extern crate drone_macros;
extern crate futures;

#[cfg(feature = "std")]
extern crate core;

pub mod heap;
pub mod mem;
pub mod prelude;
pub mod reg;
pub mod sync;
pub mod thread;

pub use drone_macros::{heap, reg, reg_block};

#[prelude_import]
#[allow(unused_imports)]
use prelude::*;
