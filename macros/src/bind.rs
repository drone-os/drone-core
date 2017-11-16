use errors::*;
use proc_macro::TokenStream;
use quote::Tokens;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::*;
use syn::{parse_token_trees, Token, TokenTree};

static USED: AtomicBool = AtomicBool::new(false);

pub(crate) fn bind(input: TokenStream) -> Result<Tokens> {
  if USED.swap(true, Relaxed) {
    bail!("The macro must be used no more than once");
  }
  let mut input = parse_token_trees(&input.to_string())?.into_iter().fuse();
  let mut names = Vec::new();
  let mut regs = Vec::new();
  'outer: loop {
    match input.next() {
      Some(TokenTree::Token(Token::Ident(name))) => {
        match input.next() {
          Some(TokenTree::Token(Token::Colon)) => (),
          token => bail!("Invalid token after `{}`: {:?}", name, token),
        }
        let mut reg = Vec::new();
        loop {
          match input.next() {
            Some(TokenTree::Token(token @ Token::Ident(_))) |
            Some(TokenTree::Token(token @ Token::ModSep)) |
            Some(TokenTree::Token(token @ Token::Lt)) |
            Some(TokenTree::Token(token @ Token::Gt)) => reg.push(token),
            Some(TokenTree::Token(Token::Comma)) | None => break,
            token => {
              bail!("Invalid token after `{}: {:?}`: {:?}", name, reg, token)
            }
          }
        }
        names.push(name);
        regs.push(reg);
      }
      None => break,
      token => bail!("Invalid token: {:?}", token),
    }
  }

  Ok(quote! {
    #(
      #[allow(unused_mut)]
      let mut #names = unsafe {
        type Register = #(#regs)*;
        <Register as ::drone::reg::Reg<_>>::Fields::__bind().into_reg()
      };
    )*
  })
}
