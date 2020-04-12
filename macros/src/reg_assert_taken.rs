use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parenthesized,
    parse::{Parse, ParseStream, Result},
    parse_macro_input,
    punctuated::Punctuated,
    Ident, LitStr, Token,
};

#[allow(dead_code)]
enum Input {
    Lit { string: LitStr },
    Concat { strings: Vec<LitStr> },
}

impl Parse for Input {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        if input.peek(LitStr) {
            let string = input.parse()?;
            Ok(Self::Lit { string })
        } else {
            let ident = input.parse::<Ident>()?;
            if ident != "concat" {
                input.error("invalid identifier");
            }
            input.parse::<Token![!]>()?;
            let content;
            parenthesized!(content in input);
            let strings =
                content.call(Punctuated::<_, Token![,]>::parse_terminated)?.into_iter().collect();
            Ok(Self::Concat { strings })
        }
    }
}

pub fn proc_macro(input: TokenStream) -> TokenStream {
    let input = &parse_macro_input!(input);
    let name = match input {
        Input::Lit { string } => string.value(),
        Input::Concat { strings } => strings.iter().map(LitStr::value).collect::<Vec<_>>().concat(),
    };
    let ident = format_ident!("__assert_register_block_taken_{}", name);
    let expanded = quote! {
        #[no_mangle]
        fn #ident() {}
    };
    expanded.into()
}
