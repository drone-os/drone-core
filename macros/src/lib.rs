//! Drone procedural macros.
//!
//! See `drone-core` documentation for details.

#![recursion_limit = "512"]
#![deny(bare_trait_objects)]
#![warn(clippy::pedantic)]
#![allow(
  clippy::cast_possible_truncation,
  clippy::cast_possible_wrap,
  clippy::similar_names
)]

extern crate proc_macro;

mod bitfield;
mod heap;
mod init_tokens;
mod reg;
mod reg_tokens;
mod res;
mod res_map;
mod res_one;
mod static_tokens;
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
pub fn res(input: TokenStream) -> TokenStream {
  res::proc_macro(input)
}

#[proc_macro]
pub fn res_map(input: TokenStream) -> TokenStream {
  res_map::proc_macro(input)
}

#[proc_macro]
pub fn res_one(input: TokenStream) -> TokenStream {
  res_one::proc_macro(input)
}

#[proc_macro]
pub fn reg(input: TokenStream) -> TokenStream {
  reg::proc_macro(input)
}

#[proc_macro]
pub fn unsafe_reg_tokens(input: TokenStream) -> TokenStream {
  reg_tokens::proc_macro(input)
}

#[proc_macro]
pub fn unsafe_init_tokens(input: TokenStream) -> TokenStream {
  init_tokens::proc_macro(input)
}

#[proc_macro]
pub fn unsafe_static_tokens(input: TokenStream) -> TokenStream {
  static_tokens::proc_macro(input)
}

#[proc_macro]
pub fn thr(input: TokenStream) -> TokenStream {
  thr::proc_macro(input)
}
