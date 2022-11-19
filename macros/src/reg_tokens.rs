use drone_macros_core::unkeywordize;
use heck::{ToSnakeCase, ToUpperCamelCase};
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream, Result};
use syn::{braced, parse_macro_input, token, AttrStyle, Attribute, Ident, Path, Token, Visibility};

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
    skip: bool,
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
        let skip = input.parse::<Option<Token![!]>>()?.is_some();
        let ident = input.parse()?;
        let content;
        braced!(content in input);
        let mut regs = Vec::new();
        while !content.is_empty() {
            regs.push(content.parse()?);
        }
        Ok(Self { attrs, vis, ident, skip, regs })
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

pub fn proc_macro(input: TokenStream) -> TokenStream {
    let Input {
        prev_macro,
        next_macro_attrs,
        next_macro_vis,
        next_macro,
        macro_root_path,
        root_path,
        blocks,
    } = parse_macro_input!(input);
    let mut tokens = Vec::new();
    let mut prev_macro = prev_macro.map(|prev_macro| quote!(#prev_macro));
    let macro_export = matches!(next_macro_vis, Visibility::Public(_));
    let (conditional_blocks, regular_blocks) =
        blocks.into_iter().partition::<Vec<_>, _>(|block| block.attrs.iter().any(is_cfg_attr));
    for (i, block) in conditional_blocks.into_iter().enumerate() {
        let mut cfg_attrs = block.attrs.iter().filter(|attr| is_cfg_attr(attr)).collect::<Vec<_>>();
        let cfg_macro = format_ident!("__{}_cfg_{}", next_macro, i);
        let doc_hidden_attr = doc_hidden_attr();
        tokens.extend(make_macro(
            macro_root_path.as_ref(),
            &root_path,
            prev_macro.as_ref(),
            &[&doc_hidden_attr, &negate_cfg_attrs(&cfg_attrs)],
            macro_export,
            &cfg_macro,
            &[],
        ));
        cfg_attrs.push(&doc_hidden_attr);
        tokens.extend(make_macro(
            macro_root_path.as_ref(),
            &root_path,
            prev_macro.as_ref(),
            &cfg_attrs,
            macro_export,
            &cfg_macro,
            &[&block],
        ));
        prev_macro =
            Some(if macro_export { quote!($crate::#cfg_macro) } else { quote!(#cfg_macro) });
    }
    tokens.extend(make_macro(
        macro_root_path.as_ref(),
        &root_path,
        prev_macro.as_ref(),
        &next_macro_attrs.iter().collect::<Vec<_>>(),
        macro_export,
        &next_macro,
        &regular_blocks.iter().collect::<Vec<_>>(),
    ));
    quote!(#(#tokens)*).into()
}

fn make_macro(
    macro_root_path: Option<&Path>,
    root_path: &Path,
    prev_macro: Option<&TokenStream2>,
    macro_attrs: &[&Attribute],
    macro_export: bool,
    macro_ident: &Ident,
    blocks: &[&Block],
) -> Vec<TokenStream2> {
    let mut tokens = Vec::new();
    let mut defs = Vec::new();
    for Block { attrs: block_attrs, vis: block_vis, ident: block_ident, skip: block_skip, regs } in
        blocks
    {
        let block_snk = block_ident.to_string().to_snake_case();
        let block_name = format_ident!("{}", unkeywordize(&block_snk));
        let mut block_tokens = Vec::new();
        let block_attrs_non_cfg =
            block_attrs.iter().filter(|attr| !is_cfg_attr(attr)).collect::<Vec<_>>();
        for Reg { attrs: reg_attrs, ident: reg_ident, skip } in regs {
            let reg_cml = format_ident!("{}", reg_ident.to_string().to_upper_camel_case());
            let reg_snk = reg_ident.to_string().to_snake_case();
            let reg_long = format_ident!("{}_{}", block_snk, reg_snk);
            let reg_short = format_ident!("{}", unkeywordize(&reg_snk));
            if !block_skip {
                block_tokens.push(quote! {
                    pub use #root_path::#reg_long as #reg_short;
                    pub use #root_path::#reg_long::Reg as #reg_cml;
                });
            }
            if !skip {
                let macro_root_path = macro_root_path.iter();
                defs.push(quote! {
                    #(#block_attrs_non_cfg)* #(#reg_attrs)*
                    #reg_long $crate #(#macro_root_path)*::#block_name::#reg_cml;
                });
            }
        }
        if !block_skip {
            tokens.push(quote! {
                #(#block_attrs)*
                #block_vis mod #block_name {
                    #(#block_tokens)*
                }
            });
        }
    }
    let macro_vis = if macro_export { quote!(#[macro_export]) } else { quote!() };
    let macro_tokens = if let Some(prev_macro) = prev_macro {
        quote! {
            #prev_macro! {
                $(#[$attr])* index => $vis $ty;
                exclude => { $($undefs,)* };
                __extend => { #(#defs)* $($defs)* };
            }
        }
    } else {
        quote! {
            ::drone_core::reg::tokens_inner! {
                $(#[$attr])* $vis $ty
                { #(#defs)* $($defs)* }
                { $($undefs;)* }
            }
        }
    };
    tokens.push(quote! {
        #(#macro_attrs)*
        #macro_vis
        macro_rules! #macro_ident {
            (
                $(#[$attr:meta])* index => $vis:vis $ty:ident
                $(; $(exclude => { $($undefs:ident),* $(,)? })? $(;)?)?
            ) => {
                #macro_ident! {
                    $(#[$attr])* index => $vis $ty;
                    exclude => { $($($($undefs,)*)?)? };
                    __extend => {};
                }
            };
            (
                $(#[$attr:meta])* index => $vis:vis $ty:ident;
                exclude => { $($undefs:ident,)* };
                __extend => { $($defs:tt)* };
            ) => {
                #macro_tokens
            };
        }
    });
    tokens
}

fn negate_cfg_attrs(cfg_attrs: &[&Attribute]) -> Attribute {
    let cfg_attrs = cfg_attrs.iter().map(|attr| &attr.tokens).collect::<Vec<_>>();
    Attribute {
        pound_token: Token![#](Span::call_site()),
        style: AttrStyle::Outer,
        bracket_token: token::Bracket(Span::call_site()),
        path: format_ident!("cfg").into(),
        tokens: quote!((not(all(#(all #cfg_attrs),*)))),
    }
}

fn doc_hidden_attr() -> Attribute {
    Attribute {
        pound_token: Token![#](Span::call_site()),
        style: AttrStyle::Outer,
        bracket_token: token::Bracket(Span::call_site()),
        path: format_ident!("doc").into(),
        tokens: quote!((hidden)),
    }
}

fn is_cfg_attr(attr: &Attribute) -> bool {
    attr.path.leading_colon.is_none()
        && attr.path.segments.len() == 1
        && attr.path.segments.first().map_or(false, |x| x.ident == "cfg")
}
