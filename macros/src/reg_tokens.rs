use drone_macros_core::{new_ident, unkeywordize};
use inflector::Inflector;
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    braced,
    parse::{Parse, ParseStream, Result},
    parse_macro_input, Attribute, Ident, Path, Token, Visibility,
};

struct RegIndex {
    prev_macro: Option<Path>,
    next_macro_attrs: Vec<Attribute>,
    next_macro_vis: Visibility,
    next_macro: Ident,
    macro_root_path: Option<Path>,
    root_path: Path,
    blocks: Blocks,
}

struct Blocks(Vec<Block>);

struct Block {
    attrs: Vec<Attribute>,
    vis: Visibility,
    ident: Ident,
    regs: Vec<Reg>,
}

struct Reg {
    attrs: Vec<Attribute>,
    ident: Ident,
}

impl Parse for RegIndex {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let next_macro_attrs = input.call(Attribute::parse_outer)?;
        let next_macro_vis = input.parse()?;
        input.parse::<Token![macro]>()?;
        let next_macro = input.parse()?;
        input.parse::<Token![;]>()?;
        let prev_macro = if input.peek(Token![use]) {
            input.parse::<Token![use]>()?;
            input.parse::<Token![macro]>()?;
            let prev_macro = input.parse()?;
            input.parse::<Token![;]>()?;
            Some(prev_macro)
        } else {
            None
        };
        let root_path = input.parse()?;
        input.parse::<Token![;]>()?;
        let macro_root_path = if input.peek(Token![;]) {
            input.parse::<Token![;]>()?;
            None
        } else {
            let path = input.parse()?;
            input.parse::<Token![;]>()?;
            Some(path)
        };
        let blocks = input.parse()?;
        Ok(Self {
            prev_macro,
            next_macro_attrs,
            next_macro_vis,
            next_macro,
            macro_root_path,
            root_path,
            blocks,
        })
    }
}

impl Parse for Blocks {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut blocks = Vec::new();
        while !input.is_empty() {
            blocks.push(input.parse()?);
        }
        Ok(Self(blocks))
    }
}

impl Parse for Block {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let vis = input.parse()?;
        input.parse::<Token![mod]>()?;
        let ident = input.parse()?;
        let content;
        braced!(content in input);
        let mut regs = Vec::new();
        while !content.is_empty() {
            regs.push(content.parse()?);
        }
        Ok(Self {
            attrs,
            vis,
            ident,
            regs,
        })
    }
}

impl Parse for Reg {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let ident = input.parse()?;
        input.parse::<Token![;]>()?;
        Ok(Self { attrs, ident })
    }
}

pub fn proc_macro(input: TokenStream) -> TokenStream {
    let RegIndex {
        prev_macro,
        next_macro_attrs,
        next_macro_vis,
        next_macro,
        macro_root_path,
        root_path,
        blocks: Blocks(blocks),
    } = &parse_macro_input!(input as RegIndex);

    let mut tokens = Vec::new();
    let mut def_tokens = Vec::new();
    let mut ctor_tokens = Vec::new();
    for Block {
        attrs,
        vis,
        ident,
        regs,
    } in blocks
    {
        let block_snk = ident.to_string().to_snake_case();
        let block_ident = new_ident!("{}", unkeywordize(block_snk.as_str().into()));
        let mut block_tokens = Vec::new();
        for Reg { attrs, ident } in regs {
            let reg_psc = new_ident!("{}", ident.to_string().to_pascal_case());
            let reg_snk = ident.to_string().to_snake_case();
            let reg_long = new_ident!("{}_{}", block_snk, reg_snk);
            let reg_short = new_ident!("{}", unkeywordize(reg_snk.as_str().into()));
            block_tokens.push(quote! {
                pub use #root_path::#reg_long as #reg_short;
                pub use #root_path::#reg_long::Reg as #reg_psc;
            });
            def_tokens.push(quote! {
                #(#attrs)*
                #[allow(missing_docs)]
                pub #reg_long: $crate#(::#macro_root_path)*::#block_ident::#reg_psc<
                    ::drone_core::reg::Srt,
                >,
            });
            ctor_tokens.push(quote! {
                #reg_long: <
                    $crate#(::#macro_root_path)*::#block_ident::#reg_psc<_> as
                        ::drone_core::reg::Reg<_>
                >::take(),
            });
        }
        tokens.push(quote! {
            #(#attrs)*
            #vis mod #block_ident {
                #(#block_tokens)*
            }
        });
    }
    let next_macro_vis = if let Visibility::Public(_) = next_macro_vis {
        quote!(#[macro_export])
    } else {
        quote!()
    };
    let macro_tokens = match prev_macro {
        Some(prev_macro) => quote! {
            #prev_macro! {
                $(#[$attr])* $vis struct $ty;
                { #(#def_tokens)* $($def)* }
                { #(#ctor_tokens)* $($ctor)* }
            }
        },
        None => quote! {
            $(#[$attr])* $vis struct $ty {
                #(#def_tokens)* $($def)*
            }
            unsafe impl ::drone_core::token::Token for $ty {
                unsafe fn take() -> Self {
                    Self { #(#ctor_tokens)* $($ctor)* }
                }
            }
        },
    };
    tokens.push(quote! {
        #(#next_macro_attrs)*
        #next_macro_vis
        macro_rules! #next_macro {
            (
                $(#[$attr:meta])* $vis:vis struct $ty:ident;
            ) => {
                #next_macro! {
                    $(#[$attr])* $vis struct $ty;
                    {} {}
                }
            };
            (
                $(#[$attr:meta])* $vis:vis struct $ty:ident;
                { $($def:tt)* }
                { $($ctor:tt)* }
            ) => {
                #(#macro_tokens)*
            };
        }
    });

    let expanded = quote! {
        #(#tokens)*
    };
    expanded.into()
}
