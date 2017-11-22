use failure::{err_msg, Error};
use proc_macro::TokenStream;
use quote::Tokens;
use syn::{parse_token_trees, DelimToken, Delimited, Ident, Token, TokenTree};

pub(crate) fn bind(input: TokenStream) -> Result<Tokens, Error> {
  parse(
    parse_token_trees(&input.to_string()).map_err(err_msg)?,
    &mut Vec::new(),
    &mut Vec::new(),
  )
}

fn parse(
  input: Vec<TokenTree>,
  names: &mut Vec<Ident>,
  regs: &mut Vec<Vec<Token>>,
) -> Result<Tokens, Error> {
  let mut input = input.into_iter().fuse();
  loop {
    match input.next() {
      Some(TokenTree::Delimited(Delimited {
        delim: DelimToken::Paren,
        tts: tokens,
      })) => {
        if !names.is_empty() {
          Err(format_err!("Invalid token: {{"))?;
        }
        parse(tokens, names, regs)?;
        let plain = plain_output(names, regs)?;
        return tuple_output(names, plain);
      }
      Some(TokenTree::Token(mod_sep @ Token::ModSep)) => {
        let mut name = vec![mod_sep];
        return parse_struct_name(input, &mut name, names, regs);
      }
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
        Some(TokenTree::Token(mod_sep @ Token::ModSep)) => {
          let mut name = vec![Token::Ident(name), mod_sep];
          return parse_struct_name(input, &mut name, names, regs);
        }
        Some(TokenTree::Delimited(Delimited {
          delim: DelimToken::Brace,
          tts: tokens,
        })) => return parse_struct(tokens, &[Token::Ident(name)], names, regs),
        token => {
          Err(format_err!("Invalid token after `{}`: {:?}", name, token))?
        }
      },
      None => break,
      token => Err(format_err!("Invalid token: {:?}", token))?,
    }
  }
  plain_output(names, regs)
}

fn parse_struct_name<I>(
  input: I,
  name: &mut Vec<Token>,
  names: &mut Vec<Ident>,
  regs: &mut Vec<Vec<Token>>,
) -> Result<Tokens, Error>
where
  I: IntoIterator<Item = TokenTree>,
{
  let mut input = input.into_iter();
  loop {
    match input.next() {
      Some(TokenTree::Token(token @ Token::Ident(_)))
      | Some(TokenTree::Token(token @ Token::ModSep)) => name.push(token),
      Some(TokenTree::Delimited(Delimited {
        delim: DelimToken::Brace,
        tts: tokens,
      })) => return parse_struct(tokens, &name, names, regs),
      token => {
        Err(format_err!("Invalid token after `{:?}`: {:?}", name, token))?
      }
    }
  }
}

fn parse_struct(
  tokens: Vec<TokenTree>,
  name: &[Token],
  names: &mut Vec<Ident>,
  regs: &mut Vec<Vec<Token>>,
) -> Result<Tokens, Error> {
  if !names.is_empty() {
    Err(format_err!("Invalid token after `{:?}`: {{", name))?;
  }
  parse(tokens, names, regs)?;
  let plain = plain_output(names, regs)?;
  return struct_output(name, names, plain);
}

fn plain_output(names: &[Ident], regs: &[Vec<Token>]) -> Result<Tokens, Error> {
  let regs_macro = regs
    .iter()
    .map(|chunks| {
      let mut chunks = chunks
        .iter()
        .take_while(|&x| *x != Token::Lt)
        .collect::<Vec<_>>();
      if let Some(&&Token::ModSep) = chunks.last() {
        chunks.pop();
      }
      chunks
    })
    .collect::<Vec<_>>();

  Ok(quote! {
    #(
      #[allow(unused_mut)]
      let mut #names = unsafe {
        type Register = #(#regs)*;
        #(#regs_macro)*!(Register, ::drone::reg::Reg::<_>, ::drone::reg::RegFields)
      };
    )*
  })
}

fn struct_output(
  name: &[Token],
  names: &[Ident],
  tokens: Tokens,
) -> Result<Tokens, Error> {
  Ok(quote! {
    {
      #tokens
      #(#name)* {
        #(#names,)*
      }
    }
  })
}

fn tuple_output(names: &[Ident], tokens: Tokens) -> Result<Tokens, Error> {
  Ok(quote! {
    {
      #tokens
      (
        #(#names,)*
      )
    }
  })
}
