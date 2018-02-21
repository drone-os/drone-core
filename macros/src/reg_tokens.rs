use drone_macros_core::parse_own_name;
use failure::{err_msg, Error};
use inflector::Inflector;
use proc_macro::TokenStream;
use quote::Tokens;
use std::{env, mem, vec};
use std::fs::File;
use std::io::prelude::*;
use syn::{parse_token_trees, DelimToken, Delimited, Ident, Lit, Token,
          TokenTree};

pub(crate) fn reg_tokens(input: TokenStream) -> Result<Tokens, Error> {
  let input = parse_token_trees(&input.to_string()).map_err(err_msg)?;
  let mut input = input.into_iter();
  let mut path = Vec::new();
  let mut struct_tokens = Vec::new();
  let mut impl_tokens = Vec::new();
  let (attrs, name) = parse_own_name(&mut input)?;
  let name =
    name.ok_or_else(|| format_err!("Unexpected end of macro invokation"))?;
  let mut inputs = vec![input];
  while let Some(mut input) = inputs.pop() {
    loop {
      match input.next() {
        Some(TokenTree::Token(Token::Ident(name))) => {
          if name == "include" {
            inputs.push(parse_include(&mut input)?.into_iter());
          } else {
            path.push(name);
          }
        }
        Some(TokenTree::Token(Token::ModSep)) => {}
        Some(TokenTree::Delimited(Delimited {
          delim: DelimToken::Brace,
          tts: tokens,
        })) => {
          let mut path = mem::replace(&mut path, Vec::new());
          let name = match path.pop() {
            Some(name) => name,
            None => Err(format_err!("Invalid tokens: {{ ... }}"))?,
          };
          let (mut x, mut y) = parse_block(path, name, tokens)?;
          struct_tokens.append(&mut x);
          impl_tokens.append(&mut y);
        }
        None => break,
        token => Err(format_err!("Invalid token: {:?}", token))?,
      }
    }
  }

  Ok(quote! {
    #(#attrs)*
    pub struct #name {
      #(#struct_tokens)*
    }

    impl ::drone_core::reg::RegTokens for #name {
      unsafe fn new() -> Self {
        Self {
          #(#impl_tokens)*
        }
      }
    }
  })
}

fn parse_block(
  path: Vec<Ident>,
  name: Ident,
  input: Vec<TokenTree>,
) -> Result<(Vec<Tokens>, Vec<Tokens>), Error> {
  let mut input = input.into_iter();
  let mut struct_tokens = Vec::new();
  let mut impl_tokens = Vec::new();
  let name = Ident::new(name.as_ref().to_snake_case());
  while let (attrs, Some(reg_name)) = parse_own_name(&mut input)? {
    let (x, y) = parse_reg(&path, &name, attrs, reg_name)?;
    struct_tokens.push(x);
    impl_tokens.push(y);
  }

  Ok((struct_tokens, impl_tokens))
}

fn parse_reg(
  path: &[Ident],
  block_name: &Ident,
  attrs: Vec<Tokens>,
  name: Ident,
) -> Result<(Tokens, Tokens), Error> {
  let name = Ident::new(name.as_ref().to_snake_case());
  let reg_name = Ident::new(format!("{}_{}", block_name, name));
  let mod_sep = &path.iter().map(|_| Token::ModSep).collect::<Vec<_>>();
  let prefix = &quote!(::#(#path #mod_sep)*#block_name::#name);

  Ok((
    quote! {
      #(#attrs)*
      pub #reg_name: #prefix::Reg<Srt>,
    },
    quote! {
      #reg_name: #prefix::Reg::new(),
    },
  ))
}

fn parse_include(
  input: &mut vec::IntoIter<TokenTree>,
) -> Result<Vec<TokenTree>, Error> {
  let mut env = None;
  let mut path = None;
  match input.next() {
    Some(TokenTree::Token(Token::Not)) => {}
    token => Err(format_err!("Invalid token: {:?}", token))?,
  }
  match input.next() {
    Some(TokenTree::Delimited(Delimited {
      delim: DelimToken::Paren,
      tts: tokens,
    })) => {
      let mut tokens = tokens.into_iter();
      match tokens.next() {
        Some(TokenTree::Token(Token::Ident(ref name))) if name == "concat" => {}
        token => Err(format_err!("Invalid token: {:?}", token))?,
      }
      match tokens.next() {
        Some(TokenTree::Token(Token::Not)) => {}
        token => Err(format_err!("Invalid token: {:?}", token))?,
      }
      match tokens.next() {
        Some(TokenTree::Delimited(Delimited {
          delim: DelimToken::Paren,
          tts: tokens,
        })) => {
          let mut tokens = tokens.into_iter();
          match tokens.next() {
            Some(TokenTree::Token(Token::Ident(ref name))) if name == "env" => {
            }
            token => Err(format_err!("Invalid token: {:?}", token))?,
          }
          match tokens.next() {
            Some(TokenTree::Token(Token::Not)) => {}
            token => Err(format_err!("Invalid token: {:?}", token))?,
          }
          match tokens.next() {
            Some(TokenTree::Delimited(Delimited {
              delim: DelimToken::Paren,
              tts: tokens,
            })) => {
              let mut tokens = tokens.into_iter();
              match tokens.next() {
                Some(TokenTree::Token(Token::Literal(Lit::Str(string, _)))) => {
                  env = Some(string);
                }
                token => Err(format_err!("Invalid token: {:?}", token))?,
              }
              match tokens.next() {
                None => {}
                token => Err(format_err!("Invalid token: {:?}", token))?,
              }
            }
            token => Err(format_err!("Invalid token: {:?}", token))?,
          }
          match tokens.next() {
            Some(TokenTree::Token(Token::Comma)) => {}
            token => Err(format_err!("Invalid token: {:?}", token))?,
          }
          match tokens.next() {
            Some(TokenTree::Token(Token::Literal(Lit::Str(string, _)))) => {
              path = Some(string);
            }
            token => Err(format_err!("Invalid token: {:?}", token))?,
          }
          match tokens.next() {
            None => {}
            token => Err(format_err!("Invalid token: {:?}", token))?,
          }
        }
        token => Err(format_err!("Invalid token: {:?}", token))?,
      }
      match tokens.next() {
        None => {}
        token => Err(format_err!("Invalid token: {:?}", token))?,
      }
    }
    token => Err(format_err!("Invalid token: {:?}", token))?,
  }
  match input.next() {
    Some(TokenTree::Token(Token::Semi)) => {}
    token => Err(format_err!("Invalid token: {:?}", token))?,
  }
  let path = format!("{}{}", env::var(env.unwrap())?, path.unwrap());
  let mut content = String::new();
  if let Ok(mut file) = File::open(path) {
    file.read_to_string(&mut content)?;
    Ok(parse_token_trees(&content).map_err(err_msg)?)
  } else {
    Ok(Vec::new())
  }
}
