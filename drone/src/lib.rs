//! The *Drone* Real-Time Operating System.
#![feature(lang_items)]
#![feature(linkage)]
#![no_std]
#![deny(missing_docs)]
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![cfg_attr(feature = "clippy", allow(precedence, doc_markdown))]

extern crate drone_core;
#[cfg(test)]
#[allow(unused_imports)]
#[macro_use]
extern crate drone_test;

#[cfg(feature = "stm32f1")]
extern crate drone_stm32f1 as drone_imp;
#[cfg(feature = "stm32l4")]
extern crate drone_stm32l4 as drone_imp;

pub use drone_core::{exception, memory};
pub use drone_imp::util;

#[macro_use]
pub mod itm;
pub mod reg;
pub mod panicking;
