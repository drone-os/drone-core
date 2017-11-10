//! Drone procedural macros.
//!
//! See `drone` documentation for details.
#![feature(decl_macro)]
#![feature(proc_macro)]
#![recursion_limit = "512"]
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![cfg_attr(feature = "clippy", allow(precedence, doc_markdown))]

#[macro_use]
extern crate error_chain;
extern crate inflector;
#[macro_use]
extern crate lazy_static;
extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate regex;
extern crate syn;

mod bind;
mod errors;
mod heap;
mod reg;
mod thread_local;

use errors::*;
use proc_macro::TokenStream;

#[doc(hidden)]
#[proc_macro]
pub fn bind_impl(input: TokenStream) -> TokenStream {
  tokens!(bind::bind(input))
}

#[doc(hidden)]
#[proc_macro]
pub fn heap_impl(input: TokenStream) -> TokenStream {
  tokens!(heap::heap(input))
}

#[doc(hidden)]
#[proc_macro]
pub fn reg_impl(input: TokenStream) -> TokenStream {
  tokens!(reg::reg(input))
}

#[doc(hidden)]
#[proc_macro]
pub fn thread_local_impl(input: TokenStream) -> TokenStream {
  tokens!(thread_local::thread_local(input))
}

macro tokens($tokens:expr) {
  match $tokens {
    Ok(tokens) => tokens.parse().unwrap(),
    Err(Error(ErrorKind::Msg(message), _)) => panic!(message),
    Err(_) => unreachable!(),
  }
}
