use failure::{err_msg, Error};
use inflector::Inflector;
use proc_macro::TokenStream;
use quote::Tokens;
use reserved::reserved_check;
use syn::{parse_token_trees, DelimToken, Delimited, Ident, IntTy, Lit, Token,
          TokenTree};

pub(crate) fn mappings(input: TokenStream) -> Result<Tokens, Error> {
  let input = parse_token_trees(&input.to_string()).map_err(err_msg)?;
  let mut input = input.into_iter();
  let mut attrs = Vec::new();
  let mut reg_attrs = Vec::new();
  let mut reg_tokens = Vec::new();
  let mut reg_names = Vec::new();
  let block = loop {
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
      Some(TokenTree::Token(Token::Ident(name))) => break name,
      None => Err(format_err!("Unexpected end of macro invokation"))?,
      token => Err(format_err!("Invalid token: {:?}", token))?,
    }
  };
  loop {
    match input.next() {
      Some(TokenTree::Token(Token::DocComment(ref string)))
        if string.starts_with("///") =>
      {
        let string = string.trim_left_matches("///");
        reg_attrs.push(quote!(#[doc = #string]));
      }
      Some(TokenTree::Token(Token::Pound)) => match input.next() {
        Some(TokenTree::Delimited(delimited)) => {
          reg_attrs.push(quote!(# #delimited))
        }
        token => Err(format_err!("Invalid tokens after `#`: {:?}", token))?,
      },
      Some(TokenTree::Token(Token::Ident(name))) => match input.next() {
        Some(TokenTree::Delimited(Delimited {
          delim: DelimToken::Brace,
          tts: tokens,
        })) => {
          reg_names.push(Ident::new(name.as_ref().to_pascal_case()));
          reg_tokens.push(parse_reg(reg_attrs, name, tokens)?);
          reg_attrs = Vec::new();
        }
        token => {
          Err(format_err!("Invalid tokens after `{}`: {:?}", name, token))?
        }
      },
      None => break,
      token => Err(format_err!("Invalid token: {:?}", token))?,
    }
  }
  let mod_name = Ident::new(block.as_ref().to_snake_case());
  let prefix = Ident::new(block.as_ref().to_pascal_case());
  let mod_names = reg_names
    .iter()
    .map(|_| mod_name.clone())
    .collect::<Vec<_>>();
  let reg_aliases = reg_names
    .iter()
    .map(|name| Ident::new(format!("{}{}", prefix, name)))
    .collect::<Vec<_>>();

  Ok(quote! {
    #(#attrs)*
    pub mod #mod_name {
      #(#reg_tokens)*
    }

    #(
      pub use self::#mod_names::#reg_names as #reg_aliases;
    )*
  })
}

fn parse_reg(
  attrs: Vec<Tokens>,
  name: Ident,
  input: Vec<TokenTree>,
) -> Result<Tokens, Error> {
  let mut input = input.into_iter();
  let mut trait_attrs = Vec::new();
  let mut trait_name = Vec::new();
  let mut field_attrs = Vec::new();
  let mut field_name = Vec::new();
  let mut field_field = Vec::new();
  let mut field_affix = Vec::new();
  let mut field_tokens = Vec::new();
  let address = match input.next() {
    Some(TokenTree::Token(Token::Literal(Lit::Int(
      address,
      IntTy::Unsuffixed,
    )))) => address,
    token => Err(format_err!(
      "Invalid tokens after `{:?} {{`: {:?}",
      name,
      token
    ))?,
  };
  let raw = match input.next() {
    Some(TokenTree::Token(Token::Literal(Lit::Int(
      raw,
      IntTy::Unsuffixed,
    )))) => Ident::new(format!("u{}", raw)),
    token => Err(format_err!(
      "Invalid tokens after `{}`: {:?}",
      address,
      token
    ))?,
  };
  let reset = match input.next() {
    Some(TokenTree::Token(Token::Literal(Lit::Int(
      reset,
      IntTy::Unsuffixed,
    )))) => Lit::Int(reset, IntTy::Usize),
    token => Err(format_err!("Invalid tokens after `{}`: {:?}", raw, token))?,
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
          token => Err(format_err!("Invalid tokens after `#`: {:?}", token))?,
        },
        Some(TokenTree::Token(Token::Ident(name))) => {
          if field_tokens.is_empty() {
            trait_attrs.push(attrs);
            trait_name.push(name);
          } else {
            match input.next() {
              Some(TokenTree::Delimited(Delimited {
                delim: DelimToken::Brace,
                tts,
              })) => {
                field_tokens.push(parse_field(
                  &attrs,
                  name,
                  tts,
                  &raw,
                  &mut field_affix,
                  &mut field_field,
                  &mut field_name,
                )?);
                field_attrs.push(attrs);
              }
              token => Err(format_err!(
                "Unexpected token after `{}`: {:?}",
                name,
                token
              ))?,
            }
          }
          break;
        }
        Some(TokenTree::Delimited(Delimited {
          delim: DelimToken::Brace,
          tts,
        })) => {
          let last = trait_attrs
            .pop()
            .and_then(|attrs| trait_name.pop().map(|name| (attrs, name)));
          if let Some((attrs, name)) = last {
            field_tokens.push(parse_field(
              &attrs,
              name,
              tts,
              &raw,
              &mut field_affix,
              &mut field_field,
              &mut field_name,
            )?);
            field_attrs.push(attrs);
          } else {
            Err(format_err!("Unexpected block: `{{ ... }}`"))?;
          }
          break;
        }
        None => {
          if field_name.len() == 0 {
            Err(format_err!("No fields defined"))?;
          }
          break 'outer;
        }
        token => Err(format_err!("Invalid token: {:?}", token))?,
      }
    }
  }
  let reg_name = Ident::new(name.as_ref().to_pascal_case());
  let mod_name = name.as_ref().to_snake_case();
  let mod_name = Ident::new(reserved_check(mod_name));
  let address = Lit::Int(address, IntTy::Unsuffixed);
  let attrs = &attrs;
  let field_name = &field_name;
  let field_field = &field_field;
  let field_field2 = field_field.clone();

  Ok(quote! {
    pub use self::#mod_name::Reg as #reg_name;

    #(#attrs)*
    pub mod #mod_name {
      use ::drone::reg;

      #(#field_tokens)*

      #(#attrs)*
      pub struct Reg<T: reg::RegTag> {
        #(
          #(#field_attrs)*
          pub #field_field: self::#field_name<T>,
        )*
      }

      impl<T: reg::RegTag> reg::Reg<T> for self::Reg<T> {
        type Val = self::Val;

        const ADDRESS: usize = #address;
      }

      impl<'a, T: reg::RegTag + 'a> reg::RegRef<'a, T> for self::Reg<T> {
        type Hold = self::Hold<'a, T>;
      }

      #(
        #(#trait_attrs)*
        impl<T: reg::RegTag> #trait_name<T> for self::Reg<T> {}
      )*

      impl From<self::Reg<reg::Ubt>> for self::Reg<reg::Sbt> {
        #[inline(always)]
        fn from(_reg: self::Reg<reg::Ubt>) -> Self {
          unsafe { Self { #(#field_field: self::#field_name::bind()),* } }
        }
      }

      impl From<self::Reg<reg::Sbt>> for self::Reg<reg::Fbt> {
        #[inline(always)]
        fn from(_reg: self::Reg<reg::Sbt>) -> Self {
          unsafe { Self { #(#field_field: self::#field_name::bind()),* } }
        }
      }

      impl From<self::Reg<reg::Sbt>> for self::Reg<reg::Ubt> {
        #[inline(always)]
        fn from(_reg: self::Reg<reg::Sbt>) -> Self {
          unsafe { Self { #(#field_field: self::#field_name::bind()),* } }
        }
      }

      impl From<self::Reg<reg::Fbt>> for self::Reg<reg::Cbt> {
        #[inline(always)]
        fn from(_reg: self::Reg<reg::Fbt>) -> Self {
          unsafe { Self { #(#field_field: self::#field_name::bind()),* } }
        }
      }

      impl reg::RegFork for self::Reg<reg::Fbt> {
        #[inline(always)]
        fn fork(&mut self) -> Self {
          Self { #(#field_field: self.#field_field2.fork()),* }
        }
      }

      #[cfg_attr(feature = "clippy", allow(expl_impl_clone_on_copy))]
      impl Clone for self::Reg<reg::Cbt> {
        #[inline(always)]
        fn clone(&self) -> Self {
          Self { ..*self }
        }
      }

      impl Copy for self::Reg<reg::Cbt> {}

      #(#attrs)*
      pub struct Hold<'a, T: reg::RegTag + 'a> {
        reg: &'a self::Reg<T>,
        val: self::Val,
      }

      impl<'a, T> reg::RegHold<'a, T, self::Reg<T>> for self::Hold<'a, T>
      where
        T: reg::RegTag,
      {
        #[inline(always)]
        unsafe fn new(reg: &'a self::Reg<T>, val: self::Val) -> Self {
          Self { reg, val }
        }

        #[inline(always)]
        fn val(&self) -> self::Val {
          self.val
        }

        #[inline(always)]
        fn set_val(&mut self, val: self::Val) {
          self.val = val;
        }
      }

      #(#attrs)*
      #[derive(Clone, Copy)]
      pub struct Val {
        raw: #raw,
      }

      impl reg::RegVal for self::Val {
        type Raw = #raw;

        #[inline(always)]
        unsafe fn reset() -> Self {
          Self::from_raw(#reset as #raw)
        }

        #[inline(always)]
        unsafe fn from_raw(raw: #raw) -> Self {
          Self { raw }
        }

        #[inline(always)]
        fn raw(&self) -> #raw {
          self.raw
        }

        #[inline(always)]
        fn raw_mut(&mut self) -> &mut #raw {
          &mut self.raw
        }
      }
    }
  })
}

fn parse_field(
  attrs: &[Tokens],
  name: Ident,
  input: Vec<TokenTree>,
  raw: &Ident,
  field_affix: &mut Vec<String>,
  field_field: &mut Vec<Ident>,
  field_name: &mut Vec<Ident>,
) -> Result<Tokens, Error> {
  let mut input = input.into_iter();
  let mut trait_attrs = Vec::new();
  let mut trait_name = Vec::new();
  let offset = match input.next() {
    Some(TokenTree::Token(Token::Literal(Lit::Int(
      offset,
      IntTy::Unsuffixed,
    )))) => offset,
    token => Err(format_err!("Invalid tokens after `{{`: {:?}", token))?,
  };
  let width = match input.next() {
    Some(TokenTree::Token(Token::Literal(Lit::Int(
      width,
      IntTy::Unsuffixed,
    )))) => width,
    token => Err(format_err!(
      "Invalid tokens after `{{ {:?}`: {:?}",
      offset,
      token
    ))?,
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
          token => Err(format_err!("Invalid tokens after `#`: {:?}", token))?,
        },
        Some(TokenTree::Token(Token::Ident(name))) => {
          trait_attrs.push(attrs);
          trait_name.push(name);
          break;
        }
        None => break 'outer,
        token => Err(format_err!("Invalid token: {:?}", token))?,
      }
    }
  }

  let mut impls = Vec::new();
  let affix = name.as_ref().to_snake_case();
  let field = Ident::new(reserved_check(affix.clone()));
  let name = Ident::new(name.as_ref().to_pascal_case());
  field_affix.push(affix.clone());
  field_field.push(field.clone());
  field_name.push(name.clone());
  if width == 1 {
    let set_field = Ident::new(format!("set_{}", affix));
    let clear_field = Ident::new(format!("clear_{}", affix));
    let toggle_field = Ident::new(format!("toggle_{}", affix));
    trait_attrs.push(Vec::new());
    trait_name.push(Ident::new("RegFieldBit"));
    if trait_name.iter().any(|name| name == "RRegField") {
      impls.push(quote! {
        impl<'a, T: reg::RegTag> self::Hold<'a, T> {
          #(#attrs)*
          #[inline(always)]
          pub fn #field(&self) -> bool {
            self.reg.#field.read(&self.val)
          }
        }
      });
    }
    if trait_name.iter().any(|name| name == "WRegField") {
      impls.push(quote! {
        impl<'a, T: reg::RegTag> self::Hold<'a, T> {
          #(#attrs)*
          #[inline(always)]
          pub fn #set_field(&mut self) -> &mut Self {
            self.reg.#field.set(&mut self.val);
            self
          }

          #(#attrs)*
          #[inline(always)]
          pub fn #clear_field(&mut self) -> &mut Self {
            self.reg.#field.clear(&mut self.val);
            self
          }

          #(#attrs)*
          #[inline(always)]
          pub fn #toggle_field(&mut self) -> &mut Self {
            self.reg.#field.toggle(&mut self.val);
            self
          }
        }
      });
    }
  } else {
    let write_field = Ident::new(format!("write_{}", affix));
    trait_attrs.push(Vec::new());
    trait_name.push(Ident::new("RegFieldBits"));
    if trait_name.iter().any(|name| name == "RRegField") {
      impls.push(quote! {
        impl<'a, T: reg::RegTag> self::Hold<'a, T> {
          #(#attrs)*
          #[inline(always)]
          pub fn #field(&self) -> #raw {
            self.reg.#field.read(&self.val)
          }
        }
      });
    }
    if trait_name.iter().any(|name| name == "WRegField") {
      impls.push(quote! {
        impl<'a, T: reg::RegTag> self::Hold<'a, T> {
          #(#attrs)*
          #[inline(always)]
          pub fn #write_field(&mut self, bits: #raw) -> &mut Self {
            self.reg.#field.write(&mut self.val, bits);
            self
          }
        }
      });
    }
  }
  let width = Lit::Int(width, IntTy::Unsuffixed);
  let offset = Lit::Int(offset, IntTy::Unsuffixed);
  let trait_field_name =
    trait_name.iter().map(|_| name.clone()).collect::<Vec<_>>();

  Ok(quote! {
    #(#impls)*

    #(#attrs)*
    pub struct #name<T: reg::RegTag> {
      _tag: T,
    }

    impl<T: reg::RegTag> self::#name<T> {
      #[inline(always)]
      pub(crate) unsafe fn bind() -> Self {
        Self { _tag: T::default() }
      }
    }

    impl<T: reg::RegTag> reg::RegField<T> for self::#name<T> {
      type Reg = self::Reg<T>;

      const OFFSET: usize = #offset;
      const WIDTH: usize = #width;
    }

    #(
      #(#trait_attrs)*
      impl<T: reg::RegTag> reg::#trait_name<T> for self::#trait_field_name<T> {}
    )*

    impl From<self::#name<reg::Sbt>> for self::#name<reg::Fbt> {
      #[inline(always)]
      fn from(_field: self::#name<reg::Sbt>) -> Self {
        Self { _tag: reg::Fbt::default() }
      }
    }

    impl From<self::#name<reg::Fbt>> for self::#name<reg::Cbt> {
      #[inline(always)]
      fn from(_field: self::#name<reg::Fbt>) -> Self {
        Self { _tag: reg::Cbt::default() }
      }
    }

    impl reg::RegFork for self::#name<reg::Fbt> {
      #[inline(always)]
      fn fork(&mut self) -> Self {
        Self { _tag: reg::Fbt::default() }
      }
    }

    #[cfg_attr(feature = "clippy", allow(expl_impl_clone_on_copy))]
    impl Clone for self::#name<reg::Cbt> {
      #[inline(always)]
      fn clone(&self) -> Self {
        Self { ..*self }
      }
    }

    impl Copy for self::#name<reg::Cbt> {}
  })
}
