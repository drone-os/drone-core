//! Drone procedural macros.
//!
//! See `drone-core` documentation for details.

#![allow(clippy::precedence)]
#![recursion_limit = "512"]

extern crate drone_macros_core;
#[macro_use]
extern crate if_chain;
extern crate inflector;
extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

mod bitfield;
mod heap;
mod reg;
mod thr;

use proc_macro::TokenStream;

#[proc_macro_derive(Bitfield, attributes(bitfield))]
pub fn derive_bitfield(input: TokenStream) -> TokenStream {
  bitfield::proc_macro_derive(input)
}

#[proc_macro]
pub fn heap(input: TokenStream) -> TokenStream {
  heap::proc_macro(input)
}

#[proc_macro]
pub fn reg_map(input: TokenStream) -> TokenStream {
  reg::map::proc_macro(input)
}

#[proc_macro]
pub fn reg_tokens(input: TokenStream) -> TokenStream {
  reg::tokens::proc_macro(input)
}

#[proc_macro]
pub fn thr(input: TokenStream) -> TokenStream {
  thr::proc_macro(input)
}
