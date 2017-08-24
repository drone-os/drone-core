//! *Drone* Hardware Independent Layer.
#![feature(asm)]
#![feature(specialization)]
#![feature(compiler_builtins_lib)]
#![no_std]
#![cfg_attr(feature = "cargo-clippy", allow(precedence, doc_markdown))]

extern crate compiler_builtins;
#[cfg(test)]
#[macro_use]
extern crate test;

pub mod reg;
pub mod memory;
pub mod exception;
