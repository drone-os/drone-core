use heck::{ToSnakeCase, ToUpperCamelCase};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::{braced, parse_macro_input, Attribute, Ident, Token, Type, Visibility};

struct Input {
    attrs: Vec<Attribute>,
    vis: Visibility,
    ident: Ident,
    tokens: Vec<Token>,
}

struct Token {
    attrs: Vec<Attribute>,
    ident: Ident,
    ty: Type,
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
        let attrs = input.call(Attribute::parse_outer)?;
        let ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let ty = input.parse()?;
        Ok(Self { attrs, ident, ty })
    }
}

pub fn proc_macro(input: TokenStream) -> TokenStream {
    let Input { attrs, vis, ident, tokens } = parse_macro_input!(input);
    let wrapper = format_ident!("__{}_static_tokens", ident.to_string().to_snake_case());
    let mut outer_tokens = Vec::new();
    let mut def_tokens = Vec::new();
    let mut ctor_tokens = Vec::new();
    for Token { attrs, ident, ty } in tokens {
        let wrapper = format_ident!("__{}_nested_static_tokens", ident.to_string().to_snake_case());
        let struct_ident = format_ident!("{}Token", ident.to_string().to_upper_camel_case());
        let field_ident = format_ident!("{}", ident.to_string().to_snake_case());
        outer_tokens.push(quote! {
            mod #wrapper {
                use super::*;

                #(#attrs)*
                pub struct #struct_ident(());

                unsafe impl ::drone_core::token::Token for #struct_ident {
                    #[inline]
                    unsafe fn take() -> Self {
                        #struct_ident(())
                    }
                }
            }

            #vis use #wrapper::#struct_ident;

            unsafe impl ::drone_core::token::StaticToken for #struct_ident {
                type Target = #ty;

                #[inline]
                fn get(&mut self) -> &mut Self::Target {
                    unsafe { &mut #ident }
                }

                #[inline]
                fn into_static(self) -> &'static mut Self::Target {
                    unsafe { &mut #ident }
                }
            }
        });
        def_tokens.push(quote! {
            #[allow(missing_docs)]
            pub #field_ident: #struct_ident,
        });
        ctor_tokens.push(quote! {
            #field_ident: ::drone_core::token::Token::take(),
        });
    }
    quote! {
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

        #(#outer_tokens)*
    }
    .into()
}
