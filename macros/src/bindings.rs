use failure::{err_msg, Error};
use proc_macro::TokenStream;
use quote::Tokens;
use syn::{parse_token_trees, Token, TokenTree};

pub(crate) fn bindings(input: TokenStream) -> Result<Tokens, Error> {
  let input = parse_token_trees(&input.to_string()).map_err(err_msg)?;
  let mut input = input.into_iter().fuse();
  let mut attrs = Vec::new();
  let mut names = Vec::new();
  let mut regs = Vec::new();
  loop {
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
          token => Err(format_err!("Invalid tokens after `#!`: {:?}", token))?,
        },
        token => Err(format_err!("Invalid tokens after `#`: {:?}", token))?,
      },
      Some(TokenTree::Token(Token::Ident(name))) => match input.next() {
        Some(TokenTree::Token(Token::Colon)) => {
          let mut reg = Vec::new();
          loop {
            match input.next() {
              Some(TokenTree::Token(token @ Token::Ident(_)))
              | Some(TokenTree::Token(token @ Token::ModSep))
              | Some(TokenTree::Token(token @ Token::Lt))
              | Some(TokenTree::Token(token @ Token::Gt)) => reg.push(token),
              Some(TokenTree::Token(Token::Comma)) | None => break,
              token => Err(format_err!(
                "Invalid token after `{}: {:?}`: {:?}",
                name,
                reg,
                token
              ))?,
            }
          }
          names.push(name);
          regs.push(reg);
        }
        token => {
          Err(format_err!("Invalid token after `{}`: {:?}", name, token))?
        }
      },
      None => break,
      token => Err(format_err!("Invalid token: {:?}", token))?,
    }
  }

  let mut regs_macro = Vec::new();
  let mut regs_args = Vec::new();
  for reg in &regs {
    let mut reg_macro = Vec::new();
    let mut reg_args = Vec::new();
    let mut reg = reg.into_iter();
    loop {
      match reg.next() {
        Some(&Token::Lt) | None => break,
        Some(token) => reg_macro.push(token.clone()),
      }
    }
    for token in reg {
      reg_args.push(token.clone())
    }
    reg_args.pop();
    regs_macro.push(reg_macro);
    regs_args.push(reg_args);
  }
  let names2 = names.clone();

  Ok(quote! {
    #(#attrs)*
    #[allow(missing_docs)]
    pub struct Bindings {
      #(
        pub #names: #(#regs)*,
      )*
    }

    impl Bindings {
      /// Create new register bindings.
      ///
      /// # Safety
      ///
      /// Must be called no more than once, at the very beginning of the program
      /// flow.
      pub unsafe fn new() -> Self {
        Self {
          #(
            #names2: #(#regs_macro)*!(#(#regs_args)*),
          )*
        }
      }
    }
  })
}
