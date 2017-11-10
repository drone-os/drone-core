use errors::*;
use inflector::Inflector;
use proc_macro::TokenStream;
use quote::Tokens;
use regex::Regex;
use syn::{parse_token_trees, DelimToken, Delimited, Ident, IntTy, Lit, Token,
          TokenTree};

lazy_static! {
  static ref RESERVED: Regex = Regex::new(r"(?x)
    ^ ( as | break | const | continue | crate | else | enum | extern | false |
    fn | for | if | impl | in | let | loop | match | mod | move | mut | pub |
    ref | return | Self | self | static | struct | super | trait | true | type |
    unsafe | use | where | while | abstract | alignof | become | box | do |
    final | macro | offsetof | override | priv | proc | pure | sizeof | typeof |
    unsized | virtual | yield ) $
  ").unwrap();
}

pub(crate) fn reg(input: TokenStream) -> Result<Tokens> {
  let mut input = parse_token_trees(&input.to_string())?.into_iter();
  let mut attrs = Vec::new();
  let mut trait_attrs = Vec::new();
  let mut trait_name = Vec::new();
  let mut field_attrs = Vec::new();
  let mut field_name = Vec::new();
  let mut field_offset = Vec::new();
  let mut field_width = Vec::new();
  let address = loop {
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
      Some(TokenTree::Token(
        Token::Literal(Lit::Int(address, IntTy::Unsuffixed)),
      )) => {
        break Lit::Int(address, IntTy::Usize);
      }
      token => bail!("Invalid token: {:?}", token),
    }
  };
  let raw = match input.next() {
    Some(
      TokenTree::Token(Token::Literal(Lit::Int(raw, IntTy::Unsuffixed))),
    ) => Ident::new(format!("u{}", raw)),
    token => bail!("Invalid tokens after {:?}: {:?}", address, token),
  };
  let reset = match input.next() {
    Some(
      TokenTree::Token(Token::Literal(Lit::Int(reset, IntTy::Unsuffixed))),
    ) => Lit::Int(reset, IntTy::Usize),
    token => bail!("Invalid tokens after {}: {:?}", raw, token),
  };
  let name = match input.next() {
    Some(TokenTree::Token(Token::Ident(name))) => name,
    token => bail!("Invalid tokens after {:?}: {:?}", reset, token),
  };
  'outer: loop {
    let mut attrs = Vec::new();
    loop {
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
          token => bail!("Invalid tokens after `#`: {:?}", token),
        },
        Some(TokenTree::Token(Token::Ident(name))) => {
          trait_attrs.push(attrs);
          trait_name.push(name);
          break;
        }
        Some(TokenTree::Delimited(Delimited {
          delim: DelimToken::Brace,
          tts: field_tokens,
        })) => {
          let mut field_tokens = field_tokens.into_iter();
          let last = trait_attrs
            .pop()
            .and_then(|attrs| trait_name.pop().map(|name| (attrs, name)));
          match last {
            Some((attriutes, name)) => {
              field_attrs.push(attriutes);
              field_name.push(name);
              let offset = match field_tokens.next() {
                Some(TokenTree::Token(
                  Token::Literal(Lit::Int(offset, IntTy::Unsuffixed)),
                )) => offset,
                token => bail!("Invalid tokens after `{{`: {:?}", token),
              };
              let width = match field_tokens.next() {
                Some(TokenTree::Token(
                  Token::Literal(Lit::Int(width, IntTy::Unsuffixed)),
                )) => width,
                token => {
                  bail!("Invalid tokens after `{{ {:?}`: {:?}", offset, token)
                }
              };
              if let Some(token) = field_tokens.next() {
                bail!(
                  "Invalid tokens after `{{ {:?} {:?}`: {:?}",
                  offset,
                  width,
                  token
                );
              }
              field_width.push(width);
              field_offset.push(offset);
            }
            None => bail!("Unexpected block: `{ ... }`"),
          }
          break;
        }
        None => break 'outer,
        token => bail!("Invalid token: {:?}", token),
      }
    }
  }
  let field_field = field_name
    .iter()
    .map(|x| Ident::new(reserved_check(x.as_ref().to_snake_case())))
    .collect::<Vec<_>>();
  let field_name = field_name
    .iter()
    .map(|x| Ident::new(x.as_ref().to_pascal_case()))
    .collect::<Vec<_>>();
  let reg_name = Ident::new(name.as_ref().to_pascal_case());
  let mod_name = Ident::new(reserved_check(name.as_ref().to_snake_case()));
  let attrs2 = attrs.clone();
  let attrs3 = attrs.clone();
  let attrs4 = attrs.clone();
  let field_name2 = field_name.clone();
  let field_field2 = field_field.clone();

  let field_tokens = field_attrs
    .iter()
    .zip(field_name.iter())
    .zip(field_field.iter())
    .zip(field_width.iter())
    .zip(field_offset.iter())
    .flat_map(|((((attrs, name), field), &width), offset)| {
      let mut tokens = Vec::new();
      let unprefixed_field = field.as_ref().trim_left_matches("_");
      tokens.push(quote! {
        #(#attrs)*
        pub struct #name<TFlavor>
        where
          TFlavor: reg::RegFlavor
        {
          _flavor: TFlavor,
        }

        impl<'a, TFlavor> reg::RegField<'a, TFlavor> for self::#name<TFlavor>
        where
          TFlavor: reg::RegFlavor + 'a
        {
          type Reg = self::Reg<TFlavor>;

          const OFFSET: usize = #offset as usize;
          const WIDTH: usize = #width as usize;

          #[inline]
          unsafe fn bind() -> Self {
            Self { _flavor: TFlavor::default() }
          }
        }

        #[cfg_attr(feature = "clippy", allow(expl_impl_clone_on_copy))]
        impl Clone for self::#name<reg::Cr> {
          #[inline]
          fn clone(&self) -> Self {
            Self { ..*self }
          }
        }

        impl Copy for self::#name<reg::Cr> {}
      });
      if width == 1 {
        let set_field = Ident::new(format!("set_{}", unprefixed_field));
        let clear_field = Ident::new(format!("clear_{}", unprefixed_field));
        let toggle_field = Ident::new(format!("toggle_{}", unprefixed_field));
        tokens.push(quote! {
          impl<'a, TFlavor> reg::RegFieldBit<'a, TFlavor>
          for self::#name<TFlavor>
          where
            TFlavor: reg::RegFlavor + 'a
          {
          }
        });
        if trait_name.iter().any(|name| name == "RReg") {
          tokens.push(quote! {
            impl<'a, TFlavor> self::Hold<'a, TFlavor>
            where
              TFlavor: reg::RegFlavor + 'a
            {
              #(#attrs)*
              #[inline]
              pub fn #field(&self) -> bool {
                self.reg.#field.read(self.val)
              }
            }
          });
        }
        if trait_name.iter().any(|name| name == "WReg") {
          tokens.push(quote! {
            impl<'a, TFlavor> self::Hold<'a, TFlavor>
            where
              TFlavor: reg::RegFlavor + 'a
            {
              #(#attrs)*
              #[inline]
              pub fn #set_field(&mut self) -> &mut Self {
                self.val = self.reg.#field.set(self.val);
                self
              }

              #(#attrs)*
              #[inline]
              pub fn #clear_field(&mut self) -> &mut Self {
                self.val = self.reg.#field.clear(self.val);
                self
              }

              #(#attrs)*
              #[inline]
              pub fn #toggle_field(&mut self) -> &mut Self {
                self.val = self.reg.#field.toggle(self.val);
                self
              }
            }
          });
        }
      } else {
        let set_field = Ident::new(format!("set_{}", unprefixed_field));
        tokens.push(quote! {
          impl<'a, TFlavor> reg::RegFieldBits<'a, TFlavor>
          for self::#name<TFlavor>
          where
            TFlavor: reg::RegFlavor + 'a
          {
          }
        });
        if trait_name.iter().any(|name| name == "RReg") {
          tokens.push(quote! {
            impl<'a, TFlavor> self::Hold<'a, TFlavor>
            where
              TFlavor: reg::RegFlavor + 'a
            {
              #(#attrs)*
              #[inline]
              pub fn #field(&self) -> #raw {
                self.reg.#field.read(self.val)
              }
            }
          });
        }
        if trait_name.iter().any(|name| name == "WReg") {
          tokens.push(quote! {
            impl<'a, TFlavor> self::Hold<'a, TFlavor>
            where
              TFlavor: reg::RegFlavor + 'a
            {
              #(#attrs)*
              #[inline]
              pub fn #set_field(&mut self, bits: #raw) -> &mut Self {
                self.val = self.reg.#field.write(self.val, bits);
                self
              }
            }
          });
        }
      }
      tokens
    })
    .collect::<Vec<_>>();

  Ok(quote! {
    pub use self::#mod_name::Reg as #reg_name;

    #(#attrs)*
    pub mod #mod_name {
      use ::drone::reg;

      #(#attrs2)*
      pub struct Reg<TFlavor>
      where
        TFlavor: reg::RegFlavor
      {
        _flavor: TFlavor,
        #(
          #(#field_attrs)*
          pub #field_field: self::#field_name<TFlavor>,
        )*
      }

      impl<'a, TFlavor> reg::Reg<'a, TFlavor> for self::Reg<TFlavor>
      where
        TFlavor: reg::RegFlavor + 'a
      {
        type Hold = self::Hold<'a, TFlavor>;

        const ADDRESS: usize = #address;

        #[inline]
        unsafe fn bind() -> Self {
          Self {
            _flavor: TFlavor::default(),
            #(
              #field_field2: self::#field_name2::bind(),
            )*
          }
        }
      }

      #(
        #(#trait_attrs)*
        impl<'a, TFlavor> #trait_name<'a, TFlavor> for self::Reg<TFlavor>
        where
          TFlavor: reg::RegFlavor + 'a
        {
        }
      )*

      #[cfg_attr(feature = "clippy", allow(expl_impl_clone_on_copy))]
      impl Clone for self::Reg<reg::Cr> {
        #[inline]
        fn clone(&self) -> Self {
          Self { ..*self }
        }
      }

      impl Copy for self::Reg<reg::Cr> {}

      #(#attrs3)*
      pub struct Hold<'a, TFlavor>
      where
        TFlavor: reg::RegFlavor + 'a
      {
        reg: &'a self::Reg<TFlavor>,
        val: self::Val,
      }

      impl<'a, TFlavor> reg::RegHold<'a, TFlavor, self::Reg<TFlavor>>
      for self::Hold<'a, TFlavor>
      where
        TFlavor: reg::RegFlavor + 'a
      {
        type Val = self::Val;

        #[inline]
        unsafe fn hold(reg: &'a self::Reg<TFlavor>, val: self::Val) -> Self {
          Self { reg, val }
        }

        #[inline]
        fn val(&self) -> self::Val {
          self.val
        }

        #[inline]
        fn set_val(&mut self, val: self::Val) {
          self.val = val;
        }
      }

      #(#attrs4)*
      #[derive(Clone, Copy)]
      pub struct Val {
        raw: #raw,
      }

      impl reg::RegVal for self::Val {
        type Raw = #raw;

        #[inline]
        unsafe fn reset() -> Self {
          Self::from_raw(#reset as #raw)
        }

        #[inline]
        unsafe fn from_raw(raw: #raw) -> Self {
          Self { raw }
        }

        #[inline]
        fn raw(self) -> #raw {
          self.raw
        }
      }

      #(#field_tokens)*
    }
  })
}

fn reserved_check(mut name: String) -> String {
  if RESERVED.is_match(&name) {
    name.insert(0, '_');
  }
  name
}
