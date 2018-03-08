//! Drone procedural macros.
//!
//! See `drone-core` documentation for details.

#![feature(proc_macro)]
#![doc(html_root_url = "https://docs.rs/drone-core-macros/0.8.0")]
#![recursion_limit = "512"]
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![cfg_attr(feature = "clippy", allow(precedence))]

#[macro_use]
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
mod driver;
mod heap;
mod reg_map;
mod reg_tokens;
mod resource;
mod thr;

use proc_macro::TokenStream;

#[proc_macro_derive(Bitfield, attributes(bitfield))]
pub fn derive_bitfield(input: TokenStream) -> TokenStream {
  bitfield::proc_macro_derive(input)
}

#[proc_macro_derive(Driver, attributes(driver))]
pub fn derive_driver(input: TokenStream) -> TokenStream {
  driver::proc_macro_derive(input)
}

#[proc_macro]
pub fn heap(input: TokenStream) -> TokenStream {
  heap::proc_macro(input)
}

#[proc_macro]
pub fn reg_map(input: TokenStream) -> TokenStream {
  reg_map::proc_macro(input)
}

#[proc_macro]
pub fn reg_tokens(input: TokenStream) -> TokenStream {
  reg_tokens::proc_macro(input)
}

#[proc_macro_derive(Resource)]
pub fn derive_resource(input: TokenStream) -> TokenStream {
  resource::proc_macro_derive(input)
}

#[proc_macro]
pub fn thr(input: TokenStream) -> TokenStream {
  thr::proc_macro(input)
}
