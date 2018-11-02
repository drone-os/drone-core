use drone_macros_core::{unkeywordize, NewStruct};
use inflector::Inflector;
use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::parse::{Parse, ParseStream, Result};
use syn::{Attribute, Ident};

struct RegTokens {
  tokens: NewStruct,
  blocks: Blocks,
}

struct Blocks(Vec<Block>);

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
    let blocks = input.parse()?;
    Ok(Self { tokens, blocks })
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
    blocks: Blocks(blocks),
  } = parse_macro_input!(input as RegTokens);
  let rt = Ident::new(
    &format!(
      "__reg_tokens_rt_{}",
      tokens_ident.to_string().to_snake_case()
    ),
    def_site,
  );
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
        #reg_name: <#block_ident::#reg_struct<_> as #rt::Reg<_>>::new()
      });
    }
  }

  let expanded = quote! {
    mod #rt {
      extern crate drone_core;

      pub use self::drone_core::reg::{Reg, RegTokens, Srt};
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
