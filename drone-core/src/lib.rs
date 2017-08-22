//! *Drone* Hardware Independent Layer.


#![feature(asm)]
#![feature(specialization)]
#![feature(compiler_builtins_lib)]

#![no_std]

#![cfg_attr(feature = "cargo-clippy", allow(precedence, doc_markdown))]


#[cfg(test)]
#[macro_use]
extern crate test;
extern crate compiler_builtins;


pub mod reg;
pub mod memory;
pub mod exception;
