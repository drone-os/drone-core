//! *Drone* implementation for *STM32F1* microcontroller series.


#![feature(asm)]
#![feature(associated_consts)]
#![feature(const_fn)]

#![no_std]

#![cfg_attr(feature = "cargo-clippy", allow(precedence, doc_markdown))]


#[cfg(test)]
#[macro_use]
extern crate test;
#[macro_use]
extern crate drone_core;


pub mod reg;
pub mod itm;
pub mod util;
