use errors::*;
use inflector::Inflector;
use proc_macro::TokenStream;
use quote::Tokens;
use syn::{parse_token_trees, Delimited, Ident, Token, TokenTree};

pub(crate) fn reg_block(input: TokenStream) -> Result<Tokens> {
  let mut input = parse_token_trees(&input.to_string())?.into_iter();
  let mut attrs = Vec::new();
  let mut regs = Vec::new();
  let mut reg_names = Vec::new();
  let name = loop {
    match input.next() {
      Some(TokenTree::Token(Token::DocComment(ref string)))
        if string.starts_with("//!") =>
      {
        let string = string.trim_left_matches("//!");
        attrs.push(quote!(#[doc = #string]));
      }
      Some(TokenTree::Token(Token::Pound)) => match input.next() {
        Some(TokenTree::Token(Token::Not)) => match input.next() {
          Some(TokenTree::Delimited(delimited)) => {
            attrs.push(quote!(# #delimited))
          }
          token => bail!("Invalid tokens after `#!`: {:?}", token),
        },
        token => bail!("Invalid tokens after `#`: {:?}", token),
      },
      Some(TokenTree::Token(Token::Ident(name))) => break name,
      token => bail!("Invalid token: {:?}", token),
    }
  };
  loop {
    match input.next() {
      Some(TokenTree::Token(Token::Ident(ref ident))) if ident == "reg" => {
        match input.next() {
          Some(TokenTree::Token(Token::Not)) => match input.next() {
            Some(TokenTree::Delimited(Delimited { tts, .. })) => regs.push(tts),
            token => bail!("Invalid tokens after `reg!`: {:?}", token),
          },
          token => bail!("Invalid tokens after `reg`: {:?}", token),
        }
      }
      Some(TokenTree::Token(Token::Semi)) => {}
      None => break,
      token => bail!("Invalid token: {:?}", token),
    }
  }
  for reg in &regs {
    let mut name = None;
    for token in reg {
      if let TokenTree::Token(Token::Ident(ref ident)) = *token {
        name = Some(ident.as_ref().to_owned());
        break;
      }
    }
    let name = name.ok_or("Register name not found")?;
    reg_names.push(name);
  }
  let mod_name = Ident::new(name.as_ref().to_snake_case());
  let mod_names = reg_names
    .iter()
    .map(|_| mod_name.clone())
    .collect::<Vec<_>>();
  let reg_name = reg_names
    .iter()
    .map(|reg_name| Ident::new(reg_name.to_pascal_case()))
    .collect::<Vec<_>>();
  let reg_alias = reg_names
    .iter()
    .map(|reg_name| {
      Ident::new(format!("{}{}", name.as_ref().to_pascal_case(), reg_name))
    })
    .collect::<Vec<_>>();

  Ok(quote! {
    #(#attrs)*
    pub mod #mod_name {
      use ::drone::reg;
      use ::drone::reg::prelude::*;

      #(
        reg! {
          #(#regs)*
        }
      )*
    }

    #(
      pub use self::#mod_names::#reg_name as #reg_alias;
    )*
  })
}
