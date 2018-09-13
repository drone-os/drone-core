use drone_macros_core::{unkeywordize, NewStruct};
use inflector::Inflector;
use proc_macro::TokenStream;
use proc_macro2::Span;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use syn::parse::{Parse, ParseStream, Result};
use syn::{parse_str, Attribute, Ident, LitStr};

struct RegTokens {
  tokens: NewStruct,
  includes: Vec<Include>,
  blocks: Blocks,
}

struct Blocks(Vec<Block>);

struct Include {
  var: LitStr,
  path: LitStr,
}

struct Block {
  ident: Ident,
  regs: Vec<Reg>,
}

struct Reg {
  attrs: Vec<Attribute>,
  ident: Ident,
}

impl Parse for RegTokens {
  fn parse(input: ParseStream) -> Result<Self> {
    let tokens = input.parse()?;
    let mut includes = Vec::new();
    while input.peek(Ident) && input.peek2(Token![!]) {
      includes.push(input.parse()?);
    }
    let blocks = input.parse()?;
    Ok(Self {
      tokens,
      includes,
      blocks,
    })
  }
}

impl Parse for Blocks {
  fn parse(input: ParseStream) -> Result<Self> {
    let mut blocks = Vec::new();
    while !input.is_empty() {
      blocks.push(input.parse()?);
    }
    Ok(Blocks(blocks))
  }
}

impl Parse for Include {
  fn parse(input: ParseStream) -> Result<Self> {
    let ident = input.parse::<Ident>()?;
    input.parse::<Token![!]>()?;
    if ident != "include" {
      return Err(input.error("invalid macro"));
    }
    let include_content;
    parenthesized!(include_content in input);
    input.parse::<Token![;]>()?;
    let ident = include_content.parse::<Ident>()?;
    include_content.parse::<Token![!]>()?;
    if ident != "concat" {
      return Err(include_content.error("invalid macro"));
    }
    let concat_content;
    parenthesized!(concat_content in include_content);
    let ident = concat_content.parse::<Ident>()?;
    concat_content.parse::<Token![!]>()?;
    if ident != "env" {
      return Err(concat_content.error("invalid macro"));
    }
    let env_content;
    parenthesized!(env_content in concat_content);
    let var = env_content.parse()?;
    concat_content.parse::<Token![,]>()?;
    let path = concat_content.parse()?;
    Ok(Self { var, path })
  }
}

impl Parse for Block {
  fn parse(input: ParseStream) -> Result<Self> {
    let ident = input.parse()?;
    let content;
    braced!(content in input);
    let mut regs = Vec::new();
    while !content.is_empty() {
      regs.push(content.parse()?);
    }
    Ok(Self { ident, regs })
  }
}

impl Parse for Reg {
  fn parse(input: ParseStream) -> Result<Self> {
    let attrs = input.call(Attribute::parse_outer)?;
    let ident = input.parse()?;
    input.parse::<Token![;]>()?;
    Ok(Self { attrs, ident })
  }
}

pub fn proc_macro(input: TokenStream) -> TokenStream {
  let (def_site, call_site) = (Span::def_site(), Span::call_site());
  let RegTokens {
    tokens:
      NewStruct {
        attrs: tokens_attrs,
        vis: tokens_vis,
        ident: tokens_ident,
      },
    includes,
    blocks: Blocks(mut blocks),
  } = parse_macro_input!(input as RegTokens);
  let rt = Ident::new(
    &format!(
      "__reg_tokens_rt_{}",
      tokens_ident.to_string().to_snake_case()
    ),
    def_site,
  );
  include_blocks(includes, &mut blocks);
  let mut tokens_tokens = Vec::new();
  let mut tokens_ctor_tokens = Vec::new();
  for Block { ident, regs } in blocks {
    let block = ident.to_string().to_snake_case();
    let block_ident =
      Ident::new(&unkeywordize(block.as_str().into()), call_site);
    for Reg { attrs, ident } in regs {
      let reg_struct =
        Ident::new(&ident.to_string().to_pascal_case(), call_site);
      let reg_name = Ident::new(
        &format!("{}_{}", block, ident.to_string().to_snake_case()),
        call_site,
      );
      tokens_tokens.push(quote! {
        #(#attrs)*
        pub #reg_name: #block_ident::#reg_struct<#rt::Srt>
      });
      tokens_ctor_tokens.push(quote! {
        #reg_name: #block_ident::#reg_struct::new()
      });
    }
  }

  let expanded = quote! {
    mod #rt {
      extern crate drone_core;

      pub use self::drone_core::reg::{RegTokens, Srt};
    }

    #(#tokens_attrs)*
    #tokens_vis struct #tokens_ident {
      #(#tokens_tokens),*
    }

    impl #rt::RegTokens for #tokens_ident {
      unsafe fn new() -> Self {
        Self { #(#tokens_ctor_tokens,)* }
      }
    }
  };
  expanded.into()
}

fn include_blocks(includes: Vec<Include>, blocks: &mut Vec<Block>) {
  for Include { var, path } in includes {
    let path = format!("{}{}", env::var(var.value()).unwrap(), path.value());
    let mut file = File::open(path).unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    let Blocks(mut extern_blocks) = parse_str(&content).unwrap();
    blocks.append(&mut extern_blocks);
  }
}
