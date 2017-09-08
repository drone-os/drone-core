//! *Drone* is a [Real-Time Operating System][rtos] Framework.
//! [rtos]: https://en.wikipedia.org/wiki/Real-time_operating_system
#![feature(optin_builtin_traits)]
#![warn(missing_docs)]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![cfg_attr(feature = "clippy", allow(precedence, doc_markdown))]

#[macro_use]
pub mod reg;
pub mod memory;
pub mod exception;

#[cfg(test)]
use std as core;
