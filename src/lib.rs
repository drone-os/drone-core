//! *Drone* is a [Real-Time Operating System][rtos] Framework.
//! [rtos]: https://en.wikipedia.org/wiki/Real-time_operating_system
#![feature(alloc)]
#![feature(allocator_api)]
#![feature(const_atomic_ptr_new)]
#![feature(const_atomic_usize_new)]
#![feature(const_fn)]
#![feature(const_ptr_null_mut)]
#![feature(core_intrinsics)]
#![feature(generators)]
#![feature(generator_trait)]
#![feature(iterator_for_each)]
#![feature(optin_builtin_traits)]
#![feature(slice_get_slice)]
#![warn(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![cfg_attr(feature = "clippy", allow(precedence, doc_markdown))]

extern crate alloc;

pub mod prelude;
#[macro_use]
pub mod reg;
pub mod heap;
pub mod mem;
pub mod routine;
pub mod collections;

#[cfg(feature = "std")]
use std as core;
