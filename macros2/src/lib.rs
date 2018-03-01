//! Drone procedural macros.
//!
//! See `drone-core` documentation for details.

#![feature(proc_macro)]
#![doc(html_root_url = "https://docs.rs/drone-core-macros2/0.8.0")]
#![recursion_limit = "256"]
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![cfg_attr(feature = "clippy", allow(precedence))]

#[macro_use]
extern crate drone_macros2_core;
#[macro_use]
extern crate if_chain;
extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

mod bitfield;
mod heap;
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
pub fn thr(input: TokenStream) -> TokenStream {
  thr::proc_macro(input)
}
