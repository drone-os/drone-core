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
//! Add or extend `std` feature as follows:
//!
//! ```toml
//! [features]
//! std = ["drone-core/std"]
//! ```

#![feature(allocator_api)]
#![feature(alloc_layout_extra)]
#![feature(core_intrinsics)]
#![feature(exhaustive_patterns)]
#![feature(generators)]
#![feature(generator_trait)]
#![feature(lang_items)]
#![feature(marker_trait_attr)]
#![feature(negative_impls)]
#![feature(never_type)]
#![feature(never_type_fallback)]
#![feature(nonnull_slice_from_raw_parts)]
#![feature(prelude_import)]
#![feature(slice_internals)]
#![feature(slice_ptr_get)]
#![feature(slice_ptr_len)]
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
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

macro_rules! load_atomic {
    ($atomic:expr, $ordering:ident) => {{
        #[cfg(not(any(feature = "atomics", loom)))]
        {
            $atomic.load()
        }
        #[cfg(any(feature = "atomics", loom))]
        {
            $atomic.load(core::sync::atomic::Ordering::$ordering)
        }
    }};
}

macro_rules! store_atomic {
    ($atomic:expr, $value:expr, $ordering:ident) => {{
        #[cfg(not(any(feature = "atomics", loom)))]
        {
            $atomic.store($value)
        }
        #[cfg(any(feature = "atomics", loom))]
        {
            $atomic.store($value, core::sync::atomic::Ordering::$ordering)
        }
    }};
}

macro_rules! modify_atomic {
    ($atomic:expr, $ordering_read:ident, $ordering_cas:ident, | $old:ident | $new:expr) => {{
        #[cfg(not(any(feature = "atomics", loom)))]
        {
            $atomic.modify(|$old| $new)
        }
        #[cfg(any(feature = "atomics", loom))]
        loop {
            match $atomic.compare_exchange_weak(
                $old,
                $new,
                core::sync::atomic::Ordering::$ordering_cas,
                core::sync::atomic::Ordering::$ordering_read,
            ) {
                Ok(state) => break state,
                Err(state) => $old = state,
            }
        }
    }};
}

macro_rules! load_modify_atomic {
    ($atomic:expr, $ordering_read:ident, $ordering_cas:ident, | $old:ident | $new:expr) => {{
        #[cfg(not(any(feature = "atomics", loom)))]
        {
            $atomic.modify(|$old| $new)
        }
        #[cfg(any(feature = "atomics", loom))]
        {
            let mut $old = $atomic.load(core::sync::atomic::Ordering::$ordering_read);
            modify_atomic!($atomic, $ordering_read, $ordering_cas, |$old| $new)
        }
    }};
}

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

#[cfg(not(feature = "std"))]
mod lang_items;

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

#[prelude_import]
#[allow(unused_imports)]
use crate::prelude::*;

/// Re-exports for use inside macros.
#[doc(hidden)]
pub mod _rt {
    pub use ::core;
}
