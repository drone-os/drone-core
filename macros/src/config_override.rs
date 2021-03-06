use proc_macro::TokenStream;
use quote::quote;
use std::env;
use syn::{
    parse::{Parse, ParseStream, Result},
    parse_macro_input, LitStr,
};

struct Input {
    contents: LitStr,
}

impl Parse for Input {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let contents = input.parse()?;
        Ok(Self { contents })
    }
}

pub fn proc_macro(input: TokenStream) -> TokenStream {
    let Input { contents } = parse_macro_input!(input);
    env::set_var("CARGO_MANIFEST_DIR_OVERRIDE", contents.value());
    quote!().into()
}
