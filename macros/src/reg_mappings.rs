use drone_macros_core::parse_own_name;
use drone_macros_core::reserved_check;
use failure::{err_msg, Error};
use inflector::Inflector;
use proc_macro::TokenStream;
use quote::Tokens;
use syn::{parse_token_trees, DelimToken, Delimited, Ident, IntTy, Lit, Token,
          TokenTree};

pub(crate) fn reg_mappings(input: TokenStream) -> Result<Tokens, Error> {
  let input = parse_token_trees(&input.to_string()).map_err(err_msg)?;
  let mut input = input.into_iter();
  let mut reg_attrs = Vec::new();
  let mut reg_tokens = Vec::new();
  let mut reg_names = Vec::new();
  let (attrs, block) = parse_own_name(&mut input)?;
  let block =
    block.ok_or_else(|| format_err!("Unexpected end of macro invokation"))?;
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
  let mut trait_name = Vec::new();
  let mut field_attrs = Vec::new();
  let mut field_name = Vec::new();
  let mut field_field = Vec::new();
  let mut field_affix = Vec::new();
  let mut field_tokens = Vec::new();
  let address = match input.next() {
    Some(TokenTree::Token(Token::Literal(
      address @ Lit::Int(_, IntTy::Unsuffixed),
    ))) => address,
    token => Err(format_err!(
      "Invalid tokens after `{:?} {{`: {:?}",
      name,
      token
    ))?,
  };
  let bits = match input.next() {
    Some(TokenTree::Token(Token::Literal(Lit::Int(
      bits,
      IntTy::Unsuffixed,
    )))) => Ident::new(format!("u{}", bits)),
    token => Err(format_err!(
      "Invalid tokens after `{:?}`: {:?}",
      address,
      token
    ))?,
  };
  let reset = match input.next() {
    Some(TokenTree::Token(Token::Literal(
      value @ Lit::Int(_, IntTy::Unsuffixed),
    ))) => value,
    token => Err(format_err!("Invalid tokens after `{}`: {:?}", bits, token))?,
  };
  loop {
    match input.next() {
      Some(TokenTree::Token(Token::Ident(name))) => trait_name.push(name),
      Some(TokenTree::Token(Token::Semi)) => break,
      token => Err(format_err!(
        "Invalid tokens after `{} {:?}`: {:?}",
        bits,
        trait_name,
        token
      ))?,
    }
  }
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
          match input.next() {
            Some(TokenTree::Delimited(Delimited {
              delim: DelimToken::Brace,
              tts,
            })) => {
              field_tokens.push(parse_field(
                &attrs,
                name,
                tts,
                &bits,
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
  let attrs = &attrs;
  let field_name = &field_name;
  let field_field = &field_field;
  let field_field2 = field_field.clone();

  Ok(quote! {
    pub use self::#mod_name::Reg as #reg_name;

    #(#attrs)*
    pub mod #mod_name {
      use ::drone_core::reg;

      #(#field_tokens)*

      #(#attrs)*
      #[derive(Clone, Copy)]
      pub struct Reg<_T: reg::RegTag> {
        #(
          #(#field_attrs)*
          pub #field_field: self::#field_name<_T>,
        )*
      }

      impl<_T: reg::RegTag> self::Reg<_T> {
        #[inline(always)]
        pub(crate) unsafe fn new() -> Self {
          Self { #(#field_field: self::#field_name { _tag: _T::default() }),* }
        }
      }

      impl<_T: reg::RegTag> reg::Reg<_T> for self::Reg<_T> {
        type Val = self::Val;

        const ADDRESS: usize = #address;
      }

      impl<'a, _T: reg::RegTag + 'a> reg::RegRef<'a, _T> for self::Reg<_T> {
        type Hold = self::Hold<'a, _T>;
      }

      #(
        impl<_T: reg::RegTag> #trait_name<_T> for self::Reg<_T> {}
      )*

      impl From<self::Reg<reg::Urt>> for self::Reg<reg::Srt> {
        #[inline(always)]
        fn from(_reg: self::Reg<reg::Urt>) -> Self {
          unsafe { Self::new() }
        }
      }

      impl From<self::Reg<reg::Urt>> for self::Reg<reg::Frt> {
        #[inline(always)]
        fn from(_reg: self::Reg<reg::Urt>) -> Self {
          unsafe { Self::new() }
        }
      }

      impl From<self::Reg<reg::Urt>> for self::Reg<reg::Crt> {
        #[inline(always)]
        fn from(_reg: self::Reg<reg::Urt>) -> Self {
          unsafe { Self::new() }
        }
      }

      impl From<self::Reg<reg::Srt>> for self::Reg<reg::Urt> {
        #[inline(always)]
        fn from(_reg: self::Reg<reg::Srt>) -> Self {
          unsafe { Self::new() }
        }
      }

      impl From<self::Reg<reg::Srt>> for self::Reg<reg::Frt> {
        #[inline(always)]
        fn from(_reg: self::Reg<reg::Srt>) -> Self {
          unsafe { Self::new() }
        }
      }

      impl From<self::Reg<reg::Srt>> for self::Reg<reg::Crt> {
        #[inline(always)]
        fn from(_reg: self::Reg<reg::Srt>) -> Self {
          unsafe { Self::new() }
        }
      }

      impl From<self::Reg<reg::Frt>> for self::Reg<reg::Crt> {
        #[inline(always)]
        fn from(_reg: self::Reg<reg::Frt>) -> Self {
          unsafe { Self::new() }
        }
      }

      impl reg::RegFork for self::Reg<reg::Frt> {
        #[inline(always)]
        fn fork(&mut self) -> Self {
          Self { #(#field_field: self.#field_field2.fork()),* }
        }
      }

      #(#attrs)*
      pub struct Hold<'a, _T: reg::RegTag + 'a> {
        reg: &'a self::Reg<_T>,
        val: self::Val,
      }

      impl<'a, _T> reg::RegHold<'a, _T, self::Reg<_T>> for self::Hold<'a, _T>
      where
        _T: reg::RegTag,
      {
        #[inline(always)]
        unsafe fn new(reg: &'a self::Reg<_T>, val: self::Val) -> Self {
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
      #[derive(Bitfield, Clone, Copy)]
      #[bitfield(default = #reset)]
      pub struct Val(#bits);
    }
  })
}

fn parse_field(
  attrs: &[Tokens],
  name: Ident,
  input: Vec<TokenTree>,
  bits: &Ident,
  field_affix: &mut Vec<String>,
  field_field: &mut Vec<Ident>,
  field_name: &mut Vec<Ident>,
) -> Result<Tokens, Error> {
  let mut input = input.into_iter();
  let mut trait_attrs = Vec::new();
  let mut trait_name = Vec::new();
  let offset = match input.next() {
    Some(TokenTree::Token(Token::Literal(
      offset @ Lit::Int(_, IntTy::Unsuffixed),
    ))) => offset,
    token => Err(format_err!("Invalid tokens after `{{`: {:?}", token))?,
  };
  let width = match input.next() {
    Some(TokenTree::Token(Token::Literal(
      width @ Lit::Int(_, IntTy::Unsuffixed),
    ))) => width,
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
  if let Lit::Int(1, IntTy::Unsuffixed) = width {
    let set_field = Ident::new(format!("set_{}", affix));
    let clear_field = Ident::new(format!("clear_{}", affix));
    let toggle_field = Ident::new(format!("toggle_{}", affix));
    trait_attrs.push(Vec::new());
    trait_name.push(Ident::new("RegFieldBit"));
    if trait_name.iter().any(|name| name == "RRRegField") {
      impls.push(quote! {
        impl<'a, _T: reg::RegTag> self::Hold<'a, _T> {
          #(#attrs)*
          #[inline(always)]
          pub fn #field(&self) -> bool {
            self.reg.#field.read(&self.val)
          }
        }
      });
    }
    if trait_name.iter().any(|name| name == "WWRegField") {
      impls.push(quote! {
        impl<'a, _T: reg::RegTag> self::Hold<'a, _T> {
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
    if trait_name.iter().any(|name| name == "RRRegField") {
      impls.push(quote! {
        impl<'a, _T: reg::RegTag> self::Hold<'a, _T> {
          #(#attrs)*
          #[inline(always)]
          pub fn #field(&self) -> #bits {
            self.reg.#field.read(&self.val)
          }
        }
      });
    }
    if trait_name.iter().any(|name| name == "WWRegField") {
      impls.push(quote! {
        impl<'a, _T: reg::RegTag> self::Hold<'a, _T> {
          #(#attrs)*
          #[inline(always)]
          pub fn #write_field(&mut self, bits: #bits) -> &mut Self {
            self.reg.#field.write(&mut self.val, bits);
            self
          }
        }
      });
    }
  }
  let trait_field_name =
    trait_name.iter().map(|_| name.clone()).collect::<Vec<_>>();

  Ok(quote! {
    #(#impls)*

    #(#attrs)*
    #[derive(Clone, Copy)]
    pub struct #name<_T: reg::RegTag> {
      _tag: _T,
    }

    impl<_T: reg::RegTag> reg::RegField<_T> for self::#name<_T> {
      type Reg = self::Reg<_T>;

      const OFFSET: usize = #offset;
      const WIDTH: usize = #width;
    }

    #(
      #(#trait_attrs)*
      impl<_T> reg::#trait_name<_T> for self::#trait_field_name<_T>
      where
        _T: reg::RegTag,
      {
      }
    )*

    impl From<self::#name<reg::Srt>> for self::#name<reg::Frt> {
      #[inline(always)]
      fn from(_field: self::#name<reg::Srt>) -> Self {
        Self { _tag: reg::Frt::default() }
      }
    }

    impl From<self::#name<reg::Srt>> for self::#name<reg::Crt> {
      #[inline(always)]
      fn from(_field: self::#name<reg::Srt>) -> Self {
        Self { _tag: reg::Crt::default() }
      }
    }

    impl From<self::#name<reg::Frt>> for self::#name<reg::Crt> {
      #[inline(always)]
      fn from(_field: self::#name<reg::Frt>) -> Self {
        Self { _tag: reg::Crt::default() }
      }
    }

    impl reg::RegFork for self::#name<reg::Frt> {
      #[inline(always)]
      fn fork(&mut self) -> Self {
        Self { _tag: reg::Frt::default() }
      }
    }
  })
}
