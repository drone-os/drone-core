//! The core crate for Drone, an Embedded Operating System.
//!
//! # Documentation
//!
//! - [Drone Book](https://book.drone-os.com/)
//! - [API documentation](https://api.drone-os.com/drone-core/0.15/)
//!
//! # Usage
//!
//! Add the crate to your `Cargo.toml` dependencies:
//!
//! ```toml
//! [dependencies]
//! drone-core = { version = "0.15.0" }
//! ```
//!
//! Add or extend `host` feature as follows:
//!
//! ```toml
//! [features]
//! host = ["drone-core/host"]
//! ```

#![feature(allocator_api)]
#![feature(alloc_error_handler)]
#![feature(alloc_layout_extra)]
#![feature(atomic_from_mut)]
#![feature(core_intrinsics)]
#![feature(exhaustive_patterns)]
#![feature(generators)]
#![feature(generator_trait)]
#![feature(marker_trait_attr)]
#![feature(never_type)]
#![feature(nonnull_slice_from_raw_parts)]
#![feature(prelude_import)]
#![feature(slice_ptr_get)]
#![feature(sync_unsafe_cell)]
#![warn(missing_docs, unsafe_op_in_unsafe_fn)]
#![warn(clippy::pedantic)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::missing_errors_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::needless_doctest_main,
    clippy::precedence,
    clippy::type_repetition_in_bounds,
    clippy::use_self,
    clippy::used_underscore_binding
)]
#![cfg_attr(not(feature = "host"), no_std)]

extern crate alloc;

#[macro_use]
mod atomic_macros;

pub mod bitfield;
pub mod fib;
pub mod heap;
pub mod inventory;
pub mod io;
pub mod mem;
pub mod periph;
pub mod platform;
pub mod prelude;
pub mod proc_loop;
pub mod reg;
pub mod stream;
pub mod sync;
pub mod thr;
pub mod token;

#[cfg(not(feature = "host"))]
mod lang_items;

#[prelude_import]
#[allow(unused_imports)]
use crate::prelude::*;
/// Defines dynamic memory structures.
///
/// See [the module level documentation](mod@heap) for details.
#[doc(inline)]
pub use drone_core_macros::heap;
#[doc(hidden)]
pub use drone_core_macros::override_layout;
/// Defines a new generic peripheral.
///
/// See [the module level documentation](mod@periph) for details.
#[doc(inline)]
pub use drone_core_macros::periph;
/// Defines a memory-mapped register.
///
/// See [the module level documentation](mod@reg) for details.
#[doc(inline)]
pub use drone_core_macros::reg;
/// Defines Drone Stream structures.
///
/// See [the module level documentation](mod@stream) for details.
#[doc(inline)]
pub use drone_core_macros::stream;

/// Re-exports for use inside macros.
#[doc(hidden)]
pub mod _rt {
    pub use ::{core, drone_stream};
}
