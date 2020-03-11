use inflector::Inflector;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream, Result},
    parse_macro_input, Attribute, Ident, Token, Visibility,
};

struct Input {
    attrs: Vec<Attribute>,
    vis: Visibility,
    ident: Ident,
}

impl Parse for Input {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let vis = input.parse()?;
        input.parse::<Token![struct]>()?;
        let ident = input.parse()?;
        input.parse::<Option<Token![;]>>()?;
        Ok(Self { attrs, vis, ident })
    }
}

pub fn proc_macro(input: TokenStream) -> TokenStream {
    let Input { attrs, vis, ident } = parse_macro_input!(input);
    let wrapper = format_ident!("__{}_simple_token", ident.to_string().to_snake_case());
    let expanded = quote! {
        mod #wrapper {
            use super::*;

            #(#attrs)*
            pub struct #ident {
                __priv: (),
            }

            unsafe impl ::drone_core::token::Token for #ident {
                #[inline]
                unsafe fn take() -> Self {
                    Self {
                        __priv: (),
                    }
                }
            }
        }

        #vis use #wrapper::#ident;
    };
    expanded.into()
}
