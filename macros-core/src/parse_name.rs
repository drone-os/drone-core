use failure::Error;
use quote::Tokens;
use std::vec;
use syn::{Ident, Token, TokenTree};

/// Parses the following pattern:
///
/// ```text
/// /// Doc comment.
/// #[doc = "attribute"]
/// Name;
/// ```
pub fn parse_own_name(
  input: &mut vec::IntoIter<TokenTree>,
) -> Result<(Vec<Tokens>, Option<Ident>), Error> {
  let mut attrs = Vec::new();
  let name = loop {
    match input.next() {
      Some(TokenTree::Token(Token::DocComment(ref string)))
        if string.starts_with("///") =>
      {
        let string = string.trim_left_matches("///");
        attrs.push(quote!(#[doc = #string]));
      }
      Some(TokenTree::Token(Token::Pound)) => match input.next() {
        Some(TokenTree::Delimited(delimited)) => {
          attrs.push(quote!(# #delimited))
        }
        token => Err(format_err!("Invalid tokens after `#`: {:?}", token))?,
      },
      Some(TokenTree::Token(Token::Ident(name))) => match input.next() {
        Some(TokenTree::Token(Token::Semi)) | None => break Some(name),
        token => {
          Err(format_err!("Invalid token after `{}`: {:?}", name, token))?
        }
      },
      None => break None,
      token => Err(format_err!("Invalid token: {:?}", token))?,
    }
  };
  Ok((attrs, name))
}

/// Parses the following pattern:
///
/// ```text
/// Name;
/// ```
pub fn parse_extern_name(
  input: &mut vec::IntoIter<TokenTree>,
) -> Result<Option<Ident>, Error> {
  Ok(match input.next() {
    Some(TokenTree::Token(Token::Ident(name))) => match input.next() {
      Some(TokenTree::Token(Token::Semi)) | None => Some(name),
      token => Err(format_err!("Invalid token after `{}`: {:?}", name, token))?,
    },
    None => None,
    token => Err(format_err!("Invalid token: {:?}", token))?,
  })
}
