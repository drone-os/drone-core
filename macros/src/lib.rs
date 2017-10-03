//! *Drone* auxiliary macros.
#![feature(proc_macro)]
#![recursion_limit = "256"]
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![cfg_attr(feature = "clippy", allow(precedence, doc_markdown))]

extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;

mod heap;
mod reg;

use proc_macro::TokenStream;

/// Configure a heap allocator.
#[proc_macro]
pub fn heap(input: TokenStream) -> TokenStream {
  heap::heap(input)
}

/// Define a memory-mapped register.
#[proc_macro]
pub fn reg(input: TokenStream) -> TokenStream {
  reg::reg(input)
}
