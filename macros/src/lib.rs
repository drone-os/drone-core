//! Drone procedural macros.
//!
//! See `drone` documentation for details.
#![feature(proc_macro)]
#![recursion_limit = "256"]
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![cfg_attr(feature = "clippy", allow(precedence, doc_markdown))]

extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;

mod bind;
mod heap;
mod reg;
mod thread_local;

use proc_macro::TokenStream;

/// See `drone` documentation for details.
#[proc_macro]
pub fn bind_imp(input: TokenStream) -> TokenStream {
  bind::bind(input)
}

/// See `drone` documentation for details.
#[proc_macro]
pub fn heap_imp(input: TokenStream) -> TokenStream {
  heap::heap(input)
}

/// See `drone` documentation for details.
#[proc_macro]
pub fn reg_imp(input: TokenStream) -> TokenStream {
  reg::reg(input)
}
/// See `drone` documentation for details.
#[proc_macro]
pub fn thread_local_imp(input: TokenStream) -> TokenStream {
  thread_local::thread_local(input)
}
