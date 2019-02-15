use drone_macros_core::{compile_error, new_ident};
use if_chain::if_chain;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
  parenthesized,
  parse::{Parse, ParseStream, Result},
  parse_macro_input, Data, DeriveInput, Fields, Ident, IntSuffix, LitInt,
  LitStr, PathArguments, Token, Type,
};

#[derive(Default)]
struct Bitfield {
  fields: Vec<Field>,
  default: Option<LitInt>,
}

struct Field {
  ident: Ident,
  mode: Mode,
  offset: LitInt,
  width: Option<LitInt>,
  doc: Option<LitStr>,
}

enum Mode {
  Read,
  ReadWrite,
  Write,
}

impl Parse for Bitfield {
  fn parse(input: ParseStream<'_>) -> Result<Self> {
    let content;
    parenthesized!(content in input);
    let mut fields = Vec::new();
    let mut default = None;
    let mut last_comma = true;
    while last_comma && !content.is_empty() {
      if content.peek(Token![default]) {
        content.parse::<Token![default]>()?;
        content.parse::<Token![=]>()?;
        if default.is_some() {
          return Err(content.error("`default` is already defined"));
        }
        default = Some(content.parse()?);
      } else {
        fields.push(content.parse()?);
      }
      last_comma = content.parse::<Option<Token![,]>>()?.is_some();
    }
    Ok(Self {
      fields: fields.into_iter().collect(),
      default,
    })
  }
}

impl Parse for Field {
  fn parse(input: ParseStream<'_>) -> Result<Self> {
    let ident = input.parse()?;
    let content;
    parenthesized!(content in input);
    let mode = content.parse()?;
    content.parse::<Token![,]>()?;
    let offset = content.parse()?;
    let width = if content.peek(Token![,]) && content.peek2(LitInt) {
      content.parse::<Token![,]>()?;
      Some(content.parse()?)
    } else {
      None
    };
    let doc = if content.peek(Token![,]) && content.peek2(LitStr) {
      content.parse::<Token![,]>()?;
      Some(content.parse()?)
    } else {
      None
    };
    Ok(Self {
      ident,
      mode,
      offset,
      width,
      doc,
    })
  }
}

impl Parse for Mode {
  fn parse(input: ParseStream<'_>) -> Result<Self> {
    let ident = input.parse::<Ident>()?;
    if ident == "r" {
      Ok(Mode::Read)
    } else if ident == "rw" {
      Ok(Mode::ReadWrite)
    } else if ident == "w" {
      Ok(Mode::Write)
    } else {
      Err(input.error("invalid mode"))
    }
  }
}

impl Mode {
  fn is_read(&self) -> bool {
    match *self {
      Mode::Read | Mode::ReadWrite => true,
      Mode::Write => false,
    }
  }

  fn is_write(&self) -> bool {
    match *self {
      Mode::Read => false,
      Mode::ReadWrite | Mode::Write => true,
    }
  }
}

pub fn proc_macro_derive(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);
  let DeriveInput {
    attrs, ident, data, ..
  } = input;
  let bitfield = attrs.into_iter().find(|attr| {
    if_chain! {
      if attr.path.leading_colon.is_none();
      if attr.path.segments.len() <= 1;
      if let Some(x) = attr.path.segments.iter().next();
      if let PathArguments::None = x.arguments;
      then { x.ident == "bitfield" } else { false }
    }
  });
  let Bitfield { fields, default } = match bitfield {
    Some(attr) => {
      let input = attr.tts.into();
      parse_macro_input!(input as Bitfield)
    }
    None => Bitfield::default(),
  };
  let default = default
    .unwrap_or_else(|| LitInt::new(0, IntSuffix::None, Span::call_site()));
  let bits = if_chain! {
    if let Data::Struct(x) = data;
    if let Fields::Unnamed(x) = x.fields;
    if x.unnamed.len() <= 1;
    if let Some(x) = x.unnamed.into_iter().next();
    if let Type::Path(x) = x.ty;
    then {
      x
    } else {
      compile_error!(
        "Bitfield can be derived only from a tuple struct with one field",
      );
    }
  };

  let field_tokens = fields
    .into_iter()
    .flat_map(|field| {
      let mut fields = Vec::new();
      let Field {
        ident,
        mode,
        offset,
        width,
        doc,
      } = field;
      let width = width
        .unwrap_or_else(|| LitInt::new(1, IntSuffix::None, Span::call_site()));
      let mut attrs = vec![quote!(#[inline])];
      if let Some(doc) = doc {
        attrs.push(quote!(#[doc = #doc]));
      }
      let attrs = &attrs;
      if width.value() == 1 {
        if mode.is_read() {
          let read_bit = new_ident!("{}", ident);
          fields.push(quote! {
            #(#attrs)*
            pub fn #read_bit(&self) -> bool {
              unsafe {
                ::drone_core::bitfield::Bitfield::read_bit(
                  self,
                  #offset as #bits,
                )
              }
            }
          });
        }
        if mode.is_write() {
          let set_bit = new_ident!("set_{}", ident);
          let clear_bit = new_ident!("clear_{}", ident);
          let toggle_bit = new_ident!("toggle_{}", ident);
          fields.push(quote! {
            #(#attrs)*
            pub fn #set_bit(&mut self) -> &mut Self {
              unsafe {
                ::drone_core::bitfield::Bitfield::set_bit(
                  self,
                  #offset as #bits,
                );
              }
              self
            }
          });
          fields.push(quote! {
            #(#attrs)*
            pub fn #clear_bit(&mut self) -> &mut Self {
              unsafe {
                ::drone_core::bitfield::Bitfield::clear_bit(
                  self,
                  #offset as #bits,
                );
              }
              self
            }
          });
          fields.push(quote! {
            #(#attrs)*
            pub fn #toggle_bit(&mut self) -> &mut Self {
              unsafe {
                ::drone_core::bitfield::Bitfield::toggle_bit(
                  self,
                  #offset as #bits,
                );
              }
              self
            }
          });
        }
      } else {
        if mode.is_read() {
          let read_bits = new_ident!("{}", ident);
          fields.push(quote! {
            #(#attrs)*
            pub fn #read_bits(&self) -> #bits {
              unsafe {
                ::drone_core::bitfield::Bitfield::read_bits(
                  self,
                  #offset as #bits,
                  #width as #bits,
                )
              }
            }
          });
        }
        if mode.is_write() {
          let write_bits = new_ident!("write_{}", ident);
          fields.push(quote! {
            #(#attrs)*
            pub fn #write_bits(&mut self, bits: #bits) -> &mut Self {
              unsafe {
                ::drone_core::bitfield::Bitfield::write_bits(
                  self,
                  #offset as #bits,
                  #width as #bits,
                  bits,
                );
              }
              self
            }
          });
        }
      }
      fields
    })
    .collect::<Vec<_>>();

  let expanded = quote! {
    impl ::drone_core::bitfield::Bitfield for #ident {
      type Bits = #bits;

      const DEFAULT: #bits = #default;

      #[inline]
      unsafe fn from_bits(bits: #bits) -> Self {
        #ident(bits)
      }

      #[inline]
      fn bits(&self) -> #bits {
        self.0
      }

      #[inline]
      fn bits_mut(&mut self) -> &mut #bits {
        &mut self.0
      }
    }

    impl #ident {
      #(#field_tokens)*
    }
  };
  expanded.into()
}
