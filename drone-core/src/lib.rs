//! *Drone* Hardware Independent Layer.
#![feature(asm)]
#![feature(specialization)]
#![no_std]
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![cfg_attr(feature = "clippy", allow(precedence, doc_markdown))]

#[cfg(test)]
#[macro_use]
extern crate drone_test;

pub mod reg;
pub mod memory;
pub mod exception;
