//! Drone procedural macros.
//!
//! See `drone` documentation for details.

#![feature(const_atomic_bool_new)]
#![feature(proc_macro)]
#![doc(html_root_url = "https://docs.rs/drone-core-macros/0.8.0")]
#![recursion_limit = "512"]
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![cfg_attr(feature = "clippy", allow(precedence))]

#[macro_use]
extern crate drone_macros_core;
#[macro_use]
extern crate failure_dup as failure;
extern crate inflector;
extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;

mod reg_mappings;
mod reg_tokens;

use proc_macro::TokenStream;

#[doc(hidden)]
#[proc_macro]
pub fn reg_mappings(input: TokenStream) -> TokenStream {
  tokens!(reg_mappings::reg_mappings(input))
}

#[doc(hidden)]
#[proc_macro]
pub fn reg_tokens(input: TokenStream) -> TokenStream {
  tokens!(reg_tokens::reg_tokens(input))
}
