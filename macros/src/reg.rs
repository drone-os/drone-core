use errors::*;
use proc_macro::TokenStream;
use quote::Tokens;
use syn::{parse_token_trees, Ident, IntTy, Lit, Token, TokenTree};

pub(crate) fn reg(input: TokenStream) -> Result<Tokens> {
  let input = parse_token_trees(&input.to_string())?;
  let mut input = input.into_iter();
  let mut attributes = Vec::new();
  let mut trait_name = Vec::new();
  let address = loop {
    match input.next() {
      Some(TokenTree::Token(Token::DocComment(string))) => {
        let string = string.trim_left_matches("//!");
        attributes.push(quote!(#[doc = #string]));
      }
      Some(TokenTree::Token(Token::Pound)) => match input.next() {
        Some(TokenTree::Token(Token::Not)) => match input.next() {
          Some(TokenTree::Delimited(delimited)) => {
            attributes.push(quote!(# #delimited))
          }
          token => bail!("Invalid tokens after `#!`: {:?}", token),
        },
        token => bail!("Invalid tokens after `#`: {:?}", token),
      },
      Some(TokenTree::Token(
        Token::Literal(Lit::Int(address, IntTy::Unsuffixed)),
      )) => {
        break Lit::Int(address, IntTy::Usize);
      }
      token => bail!("Invalid token: {:?}", token),
    }
  };
  let value_attributes = attributes.clone();
  let raw = match input.next() {
    Some(
      TokenTree::Token(Token::Literal(Lit::Int(raw, IntTy::Unsuffixed))),
    ) => Ident::new(format!("u{}", raw)),
    token => bail!("Invalid tokens after {:?}: {:?}", address, token),
  };
  let reg = match input.next() {
    Some(TokenTree::Token(Token::Ident(reg))) => reg,
    token => bail!("Invalid tokens after {}: {:?}", raw, token),
  };
  let value = Ident::new(format!("{}Val", reg));
  for token in input {
    match token {
      TokenTree::Token(Token::Ident(name)) => trait_name.push(name),
      token => bail!("Trait name expected, got {:?}", token),
    }
  }
  let trait_reg = trait_name.iter().map(|_| reg.clone()).collect::<Vec<_>>();

  Ok(quote! {
    #(#attributes)*
    pub struct #reg<T: RegFlavor> {
      flavor: ::core::marker::PhantomData<T>,
    }

    #(#value_attributes)*
    #[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
    pub struct #value {
      value: #raw,
    }

    impl<T: RegFlavor> Reg<T> for #reg<T> {
      type Value = #value;

      const ADDRESS: usize = #address;

      #[inline]
      unsafe fn bind() -> Self {
        let flavor = ::core::marker::PhantomData;
        Self { flavor }
      }
    }

    impl From<#raw> for #value {
      #[inline]
      fn from(value: #raw) -> Self {
        Self { value }
      }
    }

    impl RegVal for #value {
      type Raw = #raw;

      #[inline]
      fn into_raw(self) -> #raw {
        self.value
      }
    }

    #[cfg_attr(feature = "clippy", allow(expl_impl_clone_on_copy))]
    impl Clone for #reg<::drone::reg::Cr> {
      #[inline]
      fn clone(&self) -> Self {
        Self { ..*self }
      }
    }

    impl Copy for #reg<::drone::reg::Cr> {}

    #(
      impl<T: RegFlavor> #trait_name<T> for #trait_reg<T> {}
    )*
  })
}
