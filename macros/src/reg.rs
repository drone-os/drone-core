use failure::{err_msg, Error};
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

pub(crate) fn reg(input: TokenStream) -> Result<Tokens, Error> {
  let input = parse_token_trees(&input.to_string()).map_err(err_msg)?;
  let mut input = input.into_iter();
  let mut attrs = Vec::new();
  let mut path = Vec::new();
  let mut block = None;
  let mut reg_attrs = Vec::new();
  let mut reg_tokens = Vec::new();
  let mut reg_names = Vec::new();
  loop {
    match input.next() {
      Some(TokenTree::Token(Token::DocComment(ref string)))
        if string.starts_with("///") =>
      {
        if block.is_none() {
          Err(format_err!("Invalid tokens: ///"))?;
        }
        let string = string.trim_left_matches("///");
        reg_attrs.push(quote!(#[doc = #string]));
      }
      Some(TokenTree::Token(Token::DocComment(ref string)))
        if string.starts_with("//!") =>
      {
        let string = string.trim_left_matches("//!");
        attrs.push(quote!(#[doc = #string]));
      }
      Some(TokenTree::Token(Token::Pound)) => match input.next() {
        Some(TokenTree::Delimited(delimited)) => {
          if block.is_none() {
            Err(format_err!("Invalid tokens: #["))?;
          }
          reg_attrs.push(quote!(# #delimited))
        }
        Some(TokenTree::Token(Token::Not)) => match input.next() {
          Some(TokenTree::Delimited(delimited)) => {
            attrs.push(quote!(# #delimited))
          }
          token => Err(format_err!("Invalid tokens after `#!`: {:?}", token))?,
        },
        token => Err(format_err!("Invalid tokens after `#`: {:?}", token))?,
      },
      Some(TokenTree::Token(Token::Ident(name))) => if block.is_none() {
        block = Some(name);
      } else {
        match input.next() {
          Some(TokenTree::Delimited(Delimited {
            delim: DelimToken::Brace,
            tts: tokens,
          })) => if let Some(ref block) = block {
            reg_names.push(Ident::new(name.as_ref().to_pascal_case()));
            reg_tokens.push(parse_reg(reg_attrs, name, tokens, block, &path)?);
            reg_attrs = Vec::new();
          } else {
            Err(format_err!("Invalid tokens: {{"))?
          },
          token => {
            Err(format_err!("Invalid tokens after `{}`: {:?}", name, token))?
          }
        }
      },
      Some(TokenTree::Token(mod_sep @ Token::ModSep)) => {
        if let Some(name) = block {
          if !reg_attrs.is_empty() || !reg_tokens.is_empty() {
            Err(format_err!("Invalid tokens: ::"))?;
          }
          path.push(Token::Ident(name));
          path.push(mod_sep);
          block = None;
        } else {
          Err(format_err!("Invalid tokens after `{:?}`: `::`", path))?;
        }
      }
      None => break,
      token => Err(format_err!(
        "Invalid tokens after `{:?} {:?}`: {:?}",
        path,
        block,
        token
      ))?,
    }
  }
  let block = block.ok_or_else(|| err_msg("Block name is not specified"))?;
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
  block: &Ident,
  path: &[Token],
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
    Some(
      TokenTree::Token(Token::Literal(Lit::Int(address, IntTy::Unsuffixed))),
    ) => address,
    token => Err(format_err!(
      "Invalid tokens after `{:?} {{`: {:?}",
      name,
      token
    ))?,
  };
  let raw = match input.next() {
    Some(
      TokenTree::Token(Token::Literal(Lit::Int(raw, IntTy::Unsuffixed))),
    ) => Ident::new(format!("u{}", raw)),
    token => Err(format_err!(
      "Invalid tokens after `{}`: {:?}",
      address,
      token
    ))?,
  };
  let reset = match input.next() {
    Some(
      TokenTree::Token(Token::Literal(Lit::Int(reset, IntTy::Unsuffixed))),
    ) => Lit::Int(reset, IntTy::Usize),
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
  let block = Ident::new(block.as_ref().to_snake_case());
  let reg_name = Ident::new(name.as_ref().to_pascal_case());
  let mod_name = name.as_ref().to_snake_case();
  let mod_name = Ident::new(reserved_check(mod_name));
  let export_name = format!("drone_reg_binding_{:X}", address);
  let address = Lit::Int(address, IntTy::Unsuffixed);
  let attrs2 = attrs.clone();
  let attrs3 = attrs.clone();
  let attrs4 = attrs.clone();
  let attrs5 = attrs.clone();
  let field_name2 = field_name.clone();
  let field_name3 = field_name.clone();
  let field_name4 = field_name.clone();
  let field_name5 = field_name.clone();
  let field_field2 = field_field.clone();
  let field_field3 = field_field.clone();
  let field_field4 = field_field.clone();
  let field_field5 = field_field.clone();
  let field_field6 = field_field.clone();
  let field_field7 = field_field.clone();

  Ok(quote! {
    pub use self::#mod_name::Reg as #reg_name;

    #(#attrs)*
    pub mod #mod_name {
      pub use self::bind as Reg;

      use ::drone::reg;

      #(#field_tokens)*

      #(#attrs2)*
      pub struct Reg<T>
      where
        T: reg::RegTag,
      {
        #(
          #(#field_attrs)*
          pub #field_field: self::#field_name<T>,
        )*
      }

      impl<T> reg::Reg<T> for self::Reg<T>
      where
        T: reg::RegTag,
      {
        type Val = self::Val;

        const ADDRESS: usize = #address;
      }

      impl<'a, T> reg::RegRef<'a, T> for self::Reg<T>
      where
        T: reg::RegTag + 'a,
      {
        type Hold = self::Hold<'a, T>;
      }

      impl reg::UReg for self::Reg<reg::Urt> {
        type UpReg = self::Reg<reg::Srt>;

        #[inline(always)]
        fn upgrade(self) -> self::Reg<reg::Srt> {
          unsafe {
            Self::UpReg {
              #(
                #field_field2: self::#field_name2::bind(),
              )*
            }
          }
        }
      }

      impl reg::SReg for self::Reg<reg::Srt> {
        type UpReg = self::Reg<reg::Drt>;

        #[inline(always)]
        fn upgrade(self) -> self::Reg<reg::Drt> {
          unsafe {
            Self::UpReg {
              #(
                #field_field3: self::#field_name3::bind(),
              )*
            }
          }
        }
      }

      impl reg::DReg for self::Reg<reg::Drt> {
        type UpReg = self::Reg<reg::Crt>;

        #[inline(always)]
        fn upgrade(self) -> self::Reg<reg::Crt> {
          unsafe {
            Self::UpReg {
              #(
                #field_field4: self::#field_name4::bind(),
              )*
            }
          }
        }

        #[inline(always)]
        fn clone(&mut self) -> Self {
          Self {
            #(
              #field_field5: self.#field_field6.clone(),
            )*
          }
        }
      }

      #(
        #(#trait_attrs)*
        impl<T> #trait_name<T> for self::Reg<T>
        where
          T: reg::RegTag,
        {
        }
      )*

      #[cfg_attr(feature = "clippy", allow(expl_impl_clone_on_copy))]
      impl Clone for self::Reg<reg::Crt> {
        #[inline(always)]
        fn clone(&self) -> Self {
          Self { ..*self }
        }
      }

      impl Copy for self::Reg<reg::Crt> {}

      #(#attrs3)*
      pub struct Hold<'a, T>
      where
        T: reg::RegTag + 'a,
      {
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

      #(#attrs4)*
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

      #(#attrs5)*
      pub macro bind($($args:ty),*) {
        {
          #[allow(dead_code)]
          #[export_name = #export_name]
          #[link_section = ".drone_reg_bindings"]
          #[linkage = "external"]
          extern "C" fn __exclusive_bind() {}

          use $crate::reg::prelude::*;
          use $crate::#(#path)*#block::#mod_name;
          #mod_name::Reg::<$($args),*> {
            #(
              #field_field7: self::#field_name5::bind(),
            )*
          }
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
    Some(
      TokenTree::Token(Token::Literal(Lit::Int(offset, IntTy::Unsuffixed))),
    ) => offset,
    token => Err(format_err!("Invalid tokens after `{{`: {:?}", token))?,
  };
  let width = match input.next() {
    Some(
      TokenTree::Token(Token::Literal(Lit::Int(width, IntTy::Unsuffixed))),
    ) => width,
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
        impl<'a, T> self::Hold<'a, T>
        where
          T: reg::RegTag,
        {
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
        impl<'a, T> self::Hold<'a, T>
        where
          T: reg::RegTag,
        {
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
        impl<'a, T> self::Hold<'a, T>
        where
          T: reg::RegTag,
        {
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
        impl<'a, T> self::Hold<'a, T>
        where
          T: reg::RegTag,
        {
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
    pub struct #name<T>
    where
      T: reg::RegTag,
    {
      _tag: T,
    }

    impl<T> reg::RegField<T> for self::#name<T>
    where
      T: reg::RegTag,
    {
      type Reg = self::Reg<T>;

      const OFFSET: usize = #offset;
      const WIDTH: usize = #width;

      #[inline(always)]
      unsafe fn bind() -> Self {
        Self { _tag: T::default() }
      }
    }

    impl reg::SRegField for self::#name<reg::Srt> {
      type UpRegField = self::#name<reg::Drt>;

      #[inline(always)]
      fn upgrade(self) -> self::#name<reg::Drt> {
        Self::UpRegField { _tag: reg::Drt::default() }
      }
    }

    impl reg::DRegField for self::#name<reg::Drt> {
      type UpRegField = self::#name<reg::Crt>;

      #[inline(always)]
      fn upgrade(self) -> self::#name<reg::Crt> {
        Self::UpRegField { _tag: reg::Crt::default() }
      }

      #[inline(always)]
      fn clone(&mut self) -> Self {
        Self { _tag: reg::Drt::default() }
      }
    }

    #(
      #(#trait_attrs)*
      impl<T> reg::#trait_name<T> for self::#trait_field_name<T>
      where
        T: reg::RegTag,
      {
      }
    )*

    #[cfg_attr(feature = "clippy", allow(expl_impl_clone_on_copy))]
    impl Clone for self::#name<reg::Crt> {
      #[inline(always)]
      fn clone(&self) -> Self {
        Self { ..*self }
      }
    }

    impl Copy for self::#name<reg::Crt> {}
  })
}

fn reserved_check(mut name: String) -> String {
  if RESERVED.is_match(&name) {
    name.insert(0, '_');
  }
  name
}
