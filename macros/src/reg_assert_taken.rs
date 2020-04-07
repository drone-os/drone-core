use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream, Result},
    parse_macro_input, Ident,
};

struct Input {
    block: Ident,
}

impl Parse for Input {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let block = input.parse()?;
        Ok(Self { block })
    }
}

pub fn proc_macro(input: TokenStream) -> TokenStream {
    let Input { block } = &parse_macro_input!(input);
    let ident = format_ident!("__assert_register_block_taken_{}", block);
    let expanded = quote! {
        #[no_mangle]
        fn #ident() {}
    };
    expanded.into()
}
