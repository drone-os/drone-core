use drone_macros_core::{unkeywordize, NewStruct};
use inflector::Inflector;
use proc_macro2::{Span, TokenStream};
use std::env;
use std::fs::File;
use std::io::prelude::*;
use syn::synom::Synom;
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

impl Synom for RegTokens {
  named!(parse -> Self, do_parse!(
    tokens: syn!(NewStruct) >>
    includes: many0!(syn!(Include)) >>
    blocks: syn!(Blocks) >>
    (RegTokens { tokens, includes, blocks })
  ));
}

impl Synom for Blocks {
  named!(parse -> Self, do_parse!(
    blocks: many0!(syn!(Block)) >>
    (Blocks(blocks))
  ));
}

#[cfg_attr(feature = "cargo-clippy", allow(redundant_closure))]
impl Synom for Include {
  named!(parse -> Self, do_parse!(
    ident: syn!(Ident) >>
    switch!(value!(ident.to_string().as_ref()),
      "include" => value!(()) |
      _ => reject!()
    ) >>
    punct!(!) >>
    parens: parens!(do_parse!(
      ident: syn!(Ident) >>
      switch!(value!(ident.to_string().as_ref()),
        "concat" => value!(()) |
        _ => reject!()
      ) >>
      punct!(!) >>
      parens: parens!(do_parse!(
        ident: syn!(Ident) >>
        switch!(value!(ident.to_string().as_ref()),
          "env" => value!(()) |
          _ => reject!()
        ) >>
        punct!(!) >>
        var: map!(parens!(syn!(LitStr)), |x| x.1) >>
        punct!(,) >>
        path: syn!(LitStr) >>
        (Include { var, path })
      )) >>
      (parens.1)
    )) >>
    punct!(;) >>
    (parens.1)
  ));
}

impl Synom for Block {
  named!(parse -> Self, do_parse!(
    ident: syn!(Ident) >>
    braces: braces!(do_parse!(
      regs: many0!(syn!(Reg)) >>
      (Block { ident, regs })
    )) >>
    (braces.1)
  ));
}

impl Synom for Reg {
  named!(parse -> Self, do_parse!(
    attrs: many0!(Attribute::parse_outer) >>
    ident: syn!(Ident) >>
    punct!(;) >>
    (Reg { attrs, ident })
  ));
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
  } = try_parse2!(call_site, input);
  let rt = Ident::new("__reg_tokens_rt", def_site);
  let new = Ident::new("new", call_site);
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
      tokens_tokens.push(quote_spanned! { def_site =>
        #(#attrs)*
        pub #reg_name: #block_ident::#reg_struct<#rt::Srt>
      });
      tokens_ctor_tokens.push(quote_spanned! { def_site =>
        #reg_name: #block_ident::#reg_struct::#new()
      });
    }
  }

  let expanded = quote_spanned! { def_site =>
    mod #rt {
      extern crate drone_core;

      pub use self::drone_core::reg::{RegTokens, Srt};
    }

    #(#tokens_attrs)*
    #tokens_vis struct #tokens_ident {
      #(#tokens_tokens),*
    }

    impl #rt::RegTokens for #tokens_ident {
      unsafe fn #new() -> Self {
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
