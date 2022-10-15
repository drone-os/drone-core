use proc_macro::TokenStream;
use quote::quote;
use std::env;
use syn::parse::{Parse, ParseStream, Result};
use syn::{parse_macro_input, LitStr};

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
    env::set_var("DRONE_LAYOUT_CONFIG", contents.value());
    quote!().into()
}
