//! *Drone* is a [Real-Time Operating System][rtos] Framework.
//! [rtos]: https://en.wikipedia.org/wiki/Real-time_operating_system
#![feature(const_fn)]
#![feature(optin_builtin_traits)]
#![warn(missing_docs)]
#![cfg_attr(feature = "alloc", feature(alloc))]
#![cfg_attr(feature = "alloc", feature(global_allocator))]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![cfg_attr(feature = "clippy", allow(precedence, doc_markdown))]

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "alloc")]
extern crate linked_list_allocator;
extern crate spin;

pub mod prelude;
#[macro_use]
pub mod reg;
pub mod sync;
pub mod mem;

#[cfg(test)]
use std as core;

#[cfg(feature = "alloc")]
use linked_list_allocator::LockedHeap;

/// Global allocator.
#[cfg(feature = "alloc")]
#[global_allocator]
pub static ALLOCATOR: LockedHeap = LockedHeap::empty();
