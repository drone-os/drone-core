use proc_macro::TokenStream;
use quote::quote;
use std::collections::BTreeMap;
use syn::parse::{Parse, ParseStream, Result};
use syn::{braced, parse_macro_input, Attribute, Ident, Path, Token, Visibility};

struct Input {
    attrs: Vec<Attribute>,
    vis: Visibility,
    ident: Ident,
    defs: Vec<Def>,
    undefs: Vec<Undef>,
}

struct Def {
    attrs: Vec<Attribute>,
    ident: Ident,
    path: Path,
}

struct Undef {
    ident: Ident,
}

impl Parse for Input {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let vis = input.parse()?;
        let ident = input.parse()?;
        let content;
        braced!(content in input);
        let mut defs = Vec::new();
        while !content.is_empty() {
            defs.push(content.parse()?);
        }
        let content;
        braced!(content in input);
        let mut undefs = Vec::new();
        while !content.is_empty() {
            undefs.push(content.parse()?);
        }
        Ok(Self { attrs, vis, ident, defs, undefs })
    }
}

impl Parse for Def {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let ident = input.parse()?;
        let path = input.parse()?;
        input.parse::<Token![;]>()?;
        Ok(Self { attrs, ident, path })
    }
}

impl Parse for Undef {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let ident = input.parse()?;
        input.parse::<Token![;]>()?;
        Ok(Self { ident })
    }
}

pub fn proc_macro(input: TokenStream) -> TokenStream {
    let Input { attrs, vis, ident, defs, undefs } = &parse_macro_input!(input);
    let mut def_tokens = BTreeMap::new();
    let mut ctor_tokens = BTreeMap::new();
    for Def { attrs, ident, path } in defs {
        let string = ident.to_string();
        def_tokens.insert(string.clone(), quote! {
            #(#attrs)*
            #[allow(missing_docs)]
            pub #ident: #path<::drone_core::reg::tag::Srt>,
        });
        ctor_tokens.insert(string.clone(), quote! {
            #(#attrs)*
            #ident: ::drone_core::token::Token::take(),
        });
    }
    for Undef { ident } in undefs {
        let ident = ident.to_string();
        def_tokens.remove(&ident);
        ctor_tokens.remove(&ident);
    }
    let def_tokens = def_tokens.values();
    let ctor_tokens = ctor_tokens.values();
    quote! {
        #(#attrs)* #vis struct #ident {
            #(#def_tokens)*
        }
        unsafe impl ::drone_core::token::Token for #ident {
            #[inline]
            unsafe fn take() -> Self {
                Self { #(#ctor_tokens)* }
            }
        }
    }
    .into()
}
