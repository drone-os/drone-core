//! Procedural macros for [drone-core].
//!
//! [drone-core]: https://github.com/drone-os/drone-core

#![warn(unsafe_op_in_unsafe_fn)]
#![warn(clippy::pedantic)]
#![allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap, clippy::similar_names)]

extern crate proc_macro;

mod bitfield;
mod heap;
mod override_layout;
mod periph;
mod periph_map;
mod periph_singular;
mod reg;
mod reg_tokens;
mod reg_tokens_inner;
mod simple_token;
mod simple_tokens;
mod static_tokens;
mod stream;
mod thr_pool;
mod thr_soft;

use proc_macro::TokenStream;

#[proc_macro_derive(Bitfield, attributes(bitfield))]
pub fn derive_bitfield(input: TokenStream) -> TokenStream {
    bitfield::proc_macro_derive(input)
}

#[proc_macro]
pub fn override_layout(input: TokenStream) -> TokenStream {
    override_layout::proc_macro(input)
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
pub fn periph_singular(input: TokenStream) -> TokenStream {
    periph_singular::proc_macro(input)
}

#[proc_macro]
pub fn reg(input: TokenStream) -> TokenStream {
    reg::proc_macro(input)
}

#[proc_macro]
pub fn reg_tokens(input: TokenStream) -> TokenStream {
    reg_tokens::proc_macro(input)
}

#[proc_macro]
pub fn reg_tokens_inner(input: TokenStream) -> TokenStream {
    reg_tokens_inner::proc_macro(input)
}

#[proc_macro]
pub fn simple_token(input: TokenStream) -> TokenStream {
    simple_token::proc_macro(input)
}

#[proc_macro]
pub fn unsafe_simple_tokens(input: TokenStream) -> TokenStream {
    simple_tokens::proc_macro(input)
}

#[proc_macro]
pub fn unsafe_static_tokens(input: TokenStream) -> TokenStream {
    static_tokens::proc_macro(input)
}

#[proc_macro]
pub fn stream(input: TokenStream) -> TokenStream {
    stream::proc_macro(input)
}

#[proc_macro]
pub fn thr_pool(input: TokenStream) -> TokenStream {
    thr_pool::proc_macro(input)
}

#[proc_macro]
pub fn thr_soft(input: TokenStream) -> TokenStream {
    thr_soft::proc_macro(input)
}
