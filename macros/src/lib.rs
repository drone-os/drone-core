//! Drone procedural macros.
//!
//! See `drone` documentation for details.
#![feature(const_atomic_bool_new)]
#![feature(decl_macro)]
#![feature(proc_macro)]
#![recursion_limit = "512"]
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![cfg_attr(feature = "clippy", allow(precedence, doc_markdown))]

#[macro_use]
extern crate drone_macros_core;
#[macro_use]
extern crate failure;
extern crate inflector;
extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;

mod reg_bindings;
mod heap;
mod reg_mappings;
mod thread_local;

use proc_macro::TokenStream;

#[doc(hidden)]
#[proc_macro]
pub fn reg_bindings(input: TokenStream) -> TokenStream {
  tokens!(reg_bindings::reg_bindings(input))
}

#[doc(hidden)]
#[proc_macro]
pub fn heap(input: TokenStream) -> TokenStream {
  tokens!(heap::heap(input))
}

#[doc(hidden)]
#[proc_macro]
pub fn reg_mappings(input: TokenStream) -> TokenStream {
  tokens!(reg_mappings::reg_mappings(input))
}

#[doc(hidden)]
#[proc_macro]
pub fn thread_local(input: TokenStream) -> TokenStream {
  tokens!(thread_local::thread_local(input))
}
