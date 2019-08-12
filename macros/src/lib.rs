//! Procedural macros for [drone-core].
//!
//! [drone-core]: https://github.com/drone-os/drone-core

#![recursion_limit = "512"]
#![deny(bare_trait_objects)]
#![deny(elided_lifetimes_in_paths)]
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
mod periph;
mod periph_map;
mod periph_one;
mod reg;
mod reg_tokens;
mod static_tokens;
mod thr;
mod unit_token;

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
pub fn periph(input: TokenStream) -> TokenStream {
    periph::proc_macro(input)
}

#[proc_macro]
pub fn periph_map(input: TokenStream) -> TokenStream {
    periph_map::proc_macro(input)
}

#[proc_macro]
pub fn periph_one(input: TokenStream) -> TokenStream {
    periph_one::proc_macro(input)
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

#[proc_macro]
pub fn unit_token(input: TokenStream) -> TokenStream {
    unit_token::proc_macro(input)
}
