//! *Drone* implementation for *STM32F1* microcontroller series.
#![feature(asm)]
#![feature(const_fn)]
#![no_std]
#![deny(missing_docs)]
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![cfg_attr(feature = "clippy", allow(precedence, doc_markdown))]

#[macro_use]
extern crate drone_core;
#[cfg(test)]
#[macro_use]
extern crate drone_test;

pub mod reg;
pub mod itm;
pub mod util;
