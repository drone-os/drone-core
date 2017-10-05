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

#[proc_macro]
pub fn heap_imp(input: TokenStream) -> TokenStream {
  heap::heap(input)
}

#[proc_macro]
pub fn reg_imp(input: TokenStream) -> TokenStream {
  reg::reg(input)
}
