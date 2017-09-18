//! *Drone* is a [Real-Time Operating System][rtos] Framework.
//! [rtos]: https://en.wikipedia.org/wiki/Real-time_operating_system
#![feature(alloc)]
#![feature(allocator_api)]
#![feature(const_fn)]
#![feature(generators)]
#![feature(generator_trait)]
#![feature(iterator_for_each)]
#![feature(optin_builtin_traits)]
#![warn(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![cfg_attr(feature = "clippy", allow(precedence, doc_markdown))]

extern crate alloc as core_alloc;
extern crate linked_list_allocator;

pub mod prelude;
#[macro_use]
pub mod reg;
#[macro_use]
pub mod alloc;
pub mod mem;
pub mod routine;
pub mod collections;

#[cfg(feature = "std")]
use std as core;
