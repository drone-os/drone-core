use drone_macros_core::unkeywordize;
use inflector::Inflector;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    braced,
    parse::{Parse, ParseStream, Result},
    parse_macro_input, Attribute, Ident, Path, Token, Visibility,
};

struct Input {
    prev_macro: Option<Path>,
    next_macro_attrs: Vec<Attribute>,
    next_macro_vis: Visibility,
    next_macro: Ident,
    macro_root_path: Option<Path>,
    root_path: Path,
    blocks: Vec<Block>,
}

struct Block {
    attrs: Vec<Attribute>,
    vis: Visibility,
    ident: Ident,
    regs: Vec<Reg>,
}

struct Reg {
    attrs: Vec<Attribute>,
    ident: Ident,
    skip: bool,
}

impl Parse for Input {
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
        input.parse::<Token![crate]>()?;
        let macro_root_path = if input.peek(Token![;]) {
            input.parse::<Token![;]>()?;
            None
        } else {
            let path = input.parse()?;
            input.parse::<Token![;]>()?;
            Some(path)
        };
        let mut blocks = Vec::new();
        while !input.is_empty() {
            blocks.push(input.parse()?);
        }
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
        Ok(Self { attrs, vis, ident, regs })
    }
}

impl Parse for Reg {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let skip = input.parse::<Option<Token![!]>>()?.is_some();
        let ident = input.parse()?;
        input.parse::<Token![;]>()?;
        Ok(Self { attrs, ident, skip })
    }
}

#[allow(clippy::cognitive_complexity, clippy::too_many_lines)]
pub fn proc_macro(input: TokenStream) -> TokenStream {
    let Input {
        prev_macro,
        next_macro_attrs,
        next_macro_vis,
        next_macro,
        macro_root_path,
        root_path,
        blocks,
    } = &parse_macro_input!(input);
    let mut tokens = Vec::new();
    let mut defs = Vec::new();
    for Block { attrs: block_attrs, vis: block_vis, ident: block_ident, regs } in blocks {
        let block_snk = block_ident.to_string().to_snake_case();
        let block_name = format_ident!("{}", unkeywordize(&block_snk));
        let mut block_tokens = Vec::new();
        for Reg { attrs: reg_attrs, ident: reg_ident, skip } in regs {
            let reg_psc = format_ident!("{}", reg_ident.to_string().to_pascal_case());
            let reg_snk = reg_ident.to_string().to_snake_case();
            let reg_long = format_ident!("{}_{}", block_snk, reg_snk);
            let reg_short = format_ident!("{}", unkeywordize(&reg_snk));
            block_tokens.push(quote! {
                pub use #root_path::#reg_long as #reg_short;
                pub use #root_path::#reg_long::Reg as #reg_psc;
            });
            if !skip {
                let macro_root_path = macro_root_path.iter();
                defs.push(quote! {
                    #(#block_attrs)* #(#reg_attrs)*
                    #reg_long $crate#(#macro_root_path)*::#block_name::#reg_psc;
                });
            }
        }
        tokens.push(quote! {
            #(#block_attrs)*
            #block_vis mod #block_name {
                #(#block_tokens)*
            }
        });
    }
    let next_macro_vis =
        if let Visibility::Public(_) = next_macro_vis { quote!(#[macro_export]) } else { quote!() };
    let macro_tokens = match prev_macro {
        Some(prev_macro) => quote! {
            #prev_macro! {
                $(#[$attr])* $vis struct $ty;
                $(!$undefs;)*
                __extend { #(#defs)* $($defs)* }
            }
        },
        None => quote! {
            ::drone_core::reg::tokens_inner! {
                $(#[$attr])* $vis struct $ty;
                { #(#defs)* $($defs)* }
                { $($undefs;)* }
            }
        },
    };
    tokens.push(quote! {
        #(#next_macro_attrs)*
        #next_macro_vis
        macro_rules! #next_macro {
            (
                $(#[$attr:meta])* $vis:vis struct $ty:ident;
                $(!$undefs:ident;)*
            ) => {
                #next_macro! {
                    $(#[$attr])* $vis struct $ty;
                    $(!$undefs;)*
                    __extend {}
                }
            };
            (
                $(#[$attr:meta])* $vis:vis struct $ty:ident;
                $(!$undefs:ident;)*
                __extend { $($defs:tt)* }
            ) => {
                #macro_tokens
            };
        }
    });
    quote!(#(#tokens)*).into()
}
