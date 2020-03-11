use inflector::Inflector;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    braced,
    parse::{Parse, ParseStream, Result},
    parse_macro_input,
    punctuated::Punctuated,
    Attribute, Ident, Token, Visibility,
};

const TOKEN_SUFFIX: &str = "Token";

struct Input {
    attrs: Vec<Attribute>,
    vis: Visibility,
    ident: Ident,
    tokens: Vec<Token>,
}

struct Token {
    name: String,
}

impl Parse for Input {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let vis = input.parse()?;
        input.parse::<Token![struct]>()?;
        let ident = input.parse()?;
        let content;
        braced!(content in input);
        let tokens =
            content.call(Punctuated::<_, Token![,]>::parse_terminated)?.into_iter().collect();
        Ok(Self { attrs, vis, ident, tokens })
    }
}

impl Parse for Token {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut name = input.parse::<Ident>()?.to_string();
        if name.ends_with(TOKEN_SUFFIX) {
            name.truncate(name.len() - TOKEN_SUFFIX.len());
        } else {
            return Err(
                input.error(format!("Expected an ident which ends with `{}`", TOKEN_SUFFIX))
            );
        }
        Ok(Self { name })
    }
}

pub fn proc_macro(input: TokenStream) -> TokenStream {
    let Input { attrs, vis, ident, tokens } = parse_macro_input!(input);
    let wrapper = format_ident!("__{}_init_tokens", ident.to_string().to_snake_case());
    let mut def_tokens = Vec::new();
    let mut ctor_tokens = Vec::new();
    for Token { name } in tokens {
        let struct_ident = format_ident!("{}Token", name);
        let field_ident = format_ident!("{}", name.to_snake_case());
        def_tokens.push(quote! {
            #[allow(missing_docs)]
            pub #field_ident: #struct_ident,
        });
        ctor_tokens.push(quote! {
            #field_ident: ::drone_core::token::Token::take(),
        });
    }
    let expanded = quote! {
        mod #wrapper {
            use super::*;

            #(#attrs)*
            pub struct #ident {
                #(#def_tokens)*
                __priv: (),
            }

            unsafe impl ::drone_core::token::Token for #ident {
                #[inline]
                unsafe fn take() -> Self {
                    Self {
                        #(#ctor_tokens)*
                        __priv: (),
                    }
                }
            }
        }

        #vis use #wrapper::#ident;
    };
    expanded.into()
}
