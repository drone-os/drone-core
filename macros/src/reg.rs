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
  let mut trait_attrs = Vec::new();
  let mut trait_name = Vec::new();
  let mut field_attrs = Vec::new();
  let mut field_name = Vec::new();
  let mut field_offset = Vec::new();
  let mut field_width = Vec::new();
  let mut field_trait_attrs = Vec::new();
  let mut field_trait_name = Vec::new();
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
      Some(TokenTree::Token(Token::Ident(block))) => break block,
      token => Err(format_err!("Invalid token: {:?}", token))?,
    }
  };
  let name = match input.next() {
    Some(TokenTree::Token(Token::Ident(name))) => name,
    token => Err(format_err!("Invalid tokens after {:?}: {:?}", block, token))?,
  };
  let address = match input.next() {
    Some(
      TokenTree::Token(Token::Literal(Lit::Int(address, IntTy::Unsuffixed))),
    ) => address,
    token => Err(format_err!("Invalid tokens after {:?}: {:?}", name, token))?,
  };
  let raw = match input.next() {
    Some(
      TokenTree::Token(Token::Literal(Lit::Int(raw, IntTy::Unsuffixed))),
    ) => Ident::new(format!("u{}", raw)),
    token => Err(format_err!("Invalid tokens after {}: {:?}", address, token))?,
  };
  let reset = match input.next() {
    Some(
      TokenTree::Token(Token::Literal(Lit::Int(reset, IntTy::Unsuffixed))),
    ) => Lit::Int(reset, IntTy::Usize),
    token => Err(format_err!("Invalid tokens after {}: {:?}", raw, token))?,
  };
  'fields: loop {
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
                  Token::Literal(offset @ Lit::Int(_, IntTy::Unsuffixed)),
                )) => offset,
                token => {
                  Err(format_err!("Invalid tokens after `{{`: {:?}", token))?
                }
              };
              let width = match field_tokens.next() {
                Some(TokenTree::Token(
                  Token::Literal(width @ Lit::Int(_, IntTy::Unsuffixed)),
                )) => width,
                token => Err(format_err!(
                  "Invalid tokens after `{{ {:?}`: {:?}",
                  offset,
                  token
                ))?,
              };
              let mut trait_attrs = Vec::new();
              let mut trait_name = Vec::new();
              'traits: loop {
                let mut attrs = Vec::new();
                loop {
                  match field_tokens.next() {
                    Some(TokenTree::Token(Token::DocComment(ref string)))
                      if string.starts_with("///") =>
                    {
                      let string = string.trim_left_matches("///");
                      attrs.push(quote!(#[doc = #string]));
                    }
                    Some(TokenTree::Token(Token::Pound)) => {
                      match input.next() {
                        Some(TokenTree::Delimited(delimited)) => {
                          attrs.push(quote!(# #delimited))
                        }
                        token => Err(
                          format_err!("Invalid tokens after `#`: {:?}", token),
                        )?,
                      }
                    }
                    Some(TokenTree::Token(Token::Ident(name))) => {
                      trait_attrs.push(attrs);
                      trait_name.push(name);
                      break;
                    }
                    None => break 'traits,
                    token => Err(format_err!("Invalid token: {:?}", token))?,
                  }
                }
              }
              field_width.push(width);
              field_offset.push(offset);
              field_trait_attrs.push(trait_attrs);
              field_trait_name.push(trait_name);
            }
            None => Err(format_err!("Unexpected block: `{{ ... }}`"))?,
          }
          break;
        }
        None => {
          if field_name.len() == 0 {
            Err(format_err!("No fields defined"))?;
          }
          break 'fields;
        }
        token => Err(format_err!("Invalid token: {:?}", token))?,
      }
    }
  }
  let block = block.as_ref().to_snake_case();
  let reg_name = Ident::new(name.as_ref().to_pascal_case());
  let mod_name = name.as_ref().to_snake_case();
  let field_field = field_name
    .iter()
    .map(|x| x.as_ref().to_snake_case())
    .collect::<Vec<_>>();
  let field_full_field = field_field
    .iter()
    .map(|x| Ident::new(format!("{}_{}_{}", block, mod_name, x)))
    .collect::<Vec<_>>();
  let field_field = field_field
    .into_iter()
    .map(|x| Ident::new(reserved_check(x)))
    .collect::<Vec<_>>();
  let field_name = field_name
    .iter()
    .map(|x| Ident::new(x.as_ref().to_pascal_case()))
    .collect::<Vec<_>>();
  let mod_name = Ident::new(reserved_check(mod_name));
  let export_name = format!("drone_reg_binding_{:X}", address);
  let address = Lit::Int(address, IntTy::Unsuffixed);
  let attrs2 = attrs.clone();
  let attrs3 = attrs.clone();
  let attrs4 = attrs.clone();
  let attrs5 = attrs.clone();
  let field_attrs2 = field_attrs.clone();
  let field_name2 = field_name.clone();
  let field_name3 = field_name.clone();
  let field_field2 = field_field.clone();
  let field_field3 = field_field.clone();
  let field_full_field2 = field_full_field.clone();
  let field_full_field3 = field_full_field.clone();
  let field_full_field4 = field_full_field.clone();

  let field_tokens = field_attrs
    .iter()
    .zip(field_name.iter())
    .zip(field_field.iter())
    .zip(field_width.iter())
    .zip(field_offset.iter())
    .zip(field_trait_attrs.into_iter())
    .zip(field_trait_name.into_iter())
    .flat_map(
      |(
        (((((attrs, name), field), width), offset), mut trait_attrs),
        mut trait_name,
      )| {
        let mut tokens = Vec::new();
        let unprefixed_field = field.as_ref().trim_left_matches("_");
        tokens.push(quote! {
          #(#attrs)*
          pub struct #name<Tag>
          where
            Tag: reg::RegTag
          {
            _tag: Tag,
          }

          impl<'a, Tag> reg::RegField<'a, Tag> for self::#name<Tag>
          where
            Tag: reg::RegTag + 'a
          {
            type Reg = self::Reg<Tag>;

            const OFFSET: usize = #offset;
            const WIDTH: usize = #width;

            #[inline(always)]
            unsafe fn __bind() -> Self {
              Self { _tag: Tag::default() }
            }
          }

          #[cfg_attr(feature = "clippy", allow(expl_impl_clone_on_copy))]
          impl Clone for self::#name<reg::Cr> {
            #[inline(always)]
            fn clone(&self) -> Self {
              Self { ..*self }
            }
          }

          impl Copy for self::#name<reg::Cr> {}
        });
        if let &Lit::Int(1, _) = width {
          let set_field = Ident::new(format!("set_{}", unprefixed_field));
          let clear_field = Ident::new(format!("clear_{}", unprefixed_field));
          let toggle_field = Ident::new(format!("toggle_{}", unprefixed_field));
          trait_attrs.push(Vec::new());
          trait_name.push(Ident::new("RegFieldBit"));
          if trait_name.iter().any(|name| name == "RRegField") {
            tokens.push(quote! {
              impl<'a, Tag> self::Hold<'a, Tag>
              where
                Tag: reg::RegTag + 'a
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
            tokens.push(quote! {
              impl<'a, Tag> self::Hold<'a, Tag>
              where
                Tag: reg::RegTag + 'a
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
          let write_field = Ident::new(format!("write_{}", unprefixed_field));
          trait_attrs.push(Vec::new());
          trait_name.push(Ident::new("RegFieldBits"));
          if trait_name.iter().any(|name| name == "RRegField") {
            tokens.push(quote! {
              impl<'a, Tag> self::Hold<'a, Tag>
              where
                Tag: reg::RegTag + 'a
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
            tokens.push(quote! {
              impl<'a, Tag> self::Hold<'a, Tag>
              where
                Tag: reg::RegTag + 'a
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
        for (trait_attrs, trait_name) in
          trait_attrs.iter().zip(trait_name.iter())
        {
          tokens.push(quote! {
            #(#trait_attrs)*
            impl<'a, Tag> reg::#trait_name<'a, Tag>
            for self::#name<Tag>
            where
              Tag: reg::RegTag + 'a
            {
            }
          });
        }
        tokens
      },
    )
    .collect::<Vec<_>>();

  Ok(quote! {
    pub use self::#mod_name::Reg as #reg_name;

    #(#attrs)*
    pub mod #mod_name {
      use ::drone::reg;

      #(#field_tokens)*

      #(#attrs2)*
      pub struct Reg<Tag>
      where
        Tag: reg::RegTag
      {
        _tag: Tag,
        #(
          #(#field_attrs)*
          pub #field_field: self::#field_name<Tag>,
        )*
      }

      impl<'a, Tag> reg::Reg<'a, Tag> for self::Reg<Tag>
      where
        Tag: reg::RegTag + 'a
      {
        type Hold = self::Hold<'a, Tag>;
        type Fields = self::Fields<Tag>;

        const ADDRESS: usize = #address;

        #[inline(always)]
        fn into_fields(self) -> self::Fields<Tag> {
          self::Fields {
            #(
              #field_full_field: self.#field_field2,
            )*
          }
        }
      }

      #(
        #(#trait_attrs)*
        impl<'a, Tag> #trait_name<'a, Tag> for self::Reg<Tag>
        where
          Tag: reg::RegTag + 'a
        {
        }
      )*

      #[cfg_attr(feature = "clippy", allow(expl_impl_clone_on_copy))]
      impl Clone for self::Reg<reg::Cr> {
        #[inline(always)]
        fn clone(&self) -> Self {
          Self { ..*self }
        }
      }

      impl Copy for self::Reg<reg::Cr> {}

      #(#attrs3)*
      pub struct Fields<Tag>
      where
        Tag: reg::RegTag
      {
        #(
          #(#field_attrs2)*
          pub #field_full_field2: self::#field_name3<Tag>,
        )*
      }

      impl<'a, Tag> reg::RegFields<'a, Tag, self::Reg<Tag>>
      for self::Fields<Tag>
      where
        Tag: reg::RegTag + 'a
      {
        #[inline(always)]
        unsafe fn __bind() -> Self {
          self::Fields {
            #(
              #field_full_field3: self::#field_name2::__bind(),
            )*
          }
        }

        #[inline(always)]
        fn into_reg(self) -> self::Reg<Tag> {
          self::Reg {
            _tag: Tag::default(),
            #(
              #field_field3: self.#field_full_field4,
            )*
          }
        }
      }

      #(#attrs4)*
      pub struct Hold<'a, Tag>
      where
        Tag: reg::RegTag + 'a
      {
        reg: &'a self::Reg<Tag>,
        val: self::Val,
      }

      impl<'a, Tag> reg::RegHold<'a, Tag, self::Reg<Tag>>
      for self::Hold<'a, Tag>
      where
        Tag: reg::RegTag + 'a
      {
        type Val = self::Val;

        #[inline(always)]
        unsafe fn hold(reg: &'a self::Reg<Tag>, val: self::Val) -> Self {
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

      #(#attrs5)*
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

      #[doc(hidden)]
      pub macro Reg($reg:ty, $drone_reg:path, $drone_reg_fields:path) {
        {
          #[allow(dead_code)]
          #[export_name = #export_name]
          #[link_section = ".drone_reg_bindings"]
          #[linkage = "external"]
          extern "C" fn __exclusive_bind() {}

          use $drone_reg_fields;
          <$reg as $drone_reg>::Fields::__bind().into_reg()
        }
      }
    }
  })
}

fn reserved_check(mut name: String) -> String {
  if RESERVED.is_match(&name) {
    name.insert(0, '_');
  }
  name
}
