//! *Drone* is a [Real-Time Operating System][rtos] Framework.
//! [rtos]: https://en.wikipedia.org/wiki/Real-time_operating_system
#![feature(alloc)]
#![feature(allocator_internals)]
#![feature(const_fn)]
#![feature(generators)]
#![feature(generator_trait)]
#![feature(global_allocator)]
#![feature(optin_builtin_traits)]
#![warn(missing_docs)]
#![cfg_attr(not(any(test, feature = "test")), default_lib_allocator)]
#![cfg_attr(not(any(test, feature = "test")), no_std)]
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![cfg_attr(feature = "clippy", allow(precedence, doc_markdown))]

extern crate alloc;
extern crate linked_list_allocator;

pub mod prelude;
pub mod routine;
#[macro_use]
pub mod reg;
pub mod mem;

use linked_list_allocator::LockedHeap;
#[cfg(any(test, feature = "test"))]
use std as core;

/// Global allocator.
#[cfg_attr(not(any(test, feature = "test")), global_allocator)]
pub static ALLOC: LockedHeap = LockedHeap::empty();
