use drone_macros_core::{parse_extern_name, parse_own_name};
use failure::{err_msg, Error};
use proc_macro::TokenStream;
use quote::Tokens;
use syn::{parse_token_trees, DelimToken, Delimited, Ident, Token, TokenTree};

pub(crate) fn thread_local(input: TokenStream) -> Result<Tokens, Error> {
  let input = parse_token_trees(&input.to_string()).map_err(err_msg)?;
  let mut input = input.into_iter();
  let mut field_visiblity = Vec::new();
  let mut field_attributes = Vec::new();
  let mut field_name = Vec::new();
  let mut field_type = Vec::new();
  let mut field_init = Vec::new();
  let (attrs, name) = parse_own_name(&mut input)?;
  let static_name = parse_extern_name(&mut input)?;
  let name =
    name.ok_or_else(|| format_err!("Unexpected end of macro invokation"))?;
  let static_name = static_name
    .ok_or_else(|| format_err!("Unexpected end of macro invokation"))?;
  'outer: loop {
    let mut public = false;
    let mut inner_attributes = Vec::new();
    loop {
      match input.next() {
        Some(TokenTree::Token(Token::DocComment(ref string)))
          if string.starts_with("///") =>
        {
          let string = string.trim_left_matches("///");
          inner_attributes.push(quote!(#[doc = #string]));
        }
        Some(TokenTree::Token(Token::Pound)) => match input.next() {
          Some(TokenTree::Delimited(delimited)) => {
            inner_attributes.push(quote!(# #delimited))
          }
          token => Err(format_err!("Invalid tokens after `#`: {:?}", token))?,
        },
        Some(TokenTree::Token(Token::Ident(ref ident))) if ident == "pub" => {
          public = true;
        }
        Some(TokenTree::Token(Token::Ident(name))) => {
          match input.next() {
            Some(TokenTree::Token(Token::Colon)) => (),
            token => {
              Err(format_err!("Invalid token after `{}`: {:?}", name, token))?
            }
          }
          let mut ty = Vec::new();
          loop {
            match input.next() {
              Some(TokenTree::Token(Token::Eq)) => break,
              Some(TokenTree::Token(token)) => ty.push(token),
              token => Err(format_err!(
                "Invalid token after `{}: {:?}`: {:?}",
                name,
                ty,
                token
              ))?,
            }
          }
          let init = match input.next() {
            Some(TokenTree::Delimited(Delimited {
              delim: DelimToken::Brace,
              tts,
            })) => tts,
            token => Err(format_err!(
              "Invalid token after `{}: {:?} =`: {:?}",
              name,
              ty,
              token
            ))?,
          };
          field_visiblity.push(if public {
            Some(Ident::new("pub"))
          } else {
            None
          });
          field_attributes.push(inner_attributes);
          field_name.push(name);
          field_type.push(ty);
          field_init.push(init);
          break;
        }
        None => break 'outer,
        token => Err(format_err!("Invalid token: {:?}", token))?,
      }
    }
  }
  let field_name = &field_name;

  Ok(quote! {
    use ::drone::thread::{Chain, TaskCell};

    #(#attrs)*
    pub struct #name {
      chain: Chain,
      task: TaskCell,
      preempted_idx: usize,
      #(
        #(#field_attributes)*
        #field_visiblity #field_name: #(#field_type)*,
      )*
    }

    impl #name {
      /// Creates a blank thread.
      #[allow(dead_code)]
      #[inline(always)]
      pub const fn new(_index: usize) -> Self {
        Self {
          chain: Chain::new(),
          task: TaskCell::new(),
          preempted_idx: 0,
          #(#field_name: { #(#field_init)* }),*
        }
      }
    }

    impl Thread for #name {
      #[inline(always)]
      fn array() -> *mut [Self] {
        unsafe { &mut #static_name }
      }

      #[inline(always)]
      fn chain(&self) -> &Chain {
        &self.chain
      }

      #[inline(always)]
      fn chain_mut(&mut self) -> &mut Chain {
        &mut self.chain
      }

      #[inline(always)]
      fn task(&self) -> &TaskCell {
        &self.task
      }

      #[inline(always)]
      fn preempted_idx(&self) -> usize {
        self.preempted_idx
      }

      #[inline(always)]
      fn set_preempted_idx(&mut self, index: usize) {
        self.preempted_idx = index;
      }
    }
  })
}
