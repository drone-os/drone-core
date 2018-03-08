use drone_macros_core::emit_err;
use inflector::Inflector;
use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{parse, Data, DeriveInput, Fields, Ident, Index, IntSuffix, LitInt,
          LitStr, PathArguments, Type};
use syn::punctuated::{Pair, Punctuated};
use syn::spanned::Spanned;
use syn::synom::Synom;

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

impl Synom for Bitfield {
  named!(parse -> Self, do_parse!(
    parens: parens!(do_parse!(
      fields: call!(Punctuated::<Field, Token![,]>::parse_separated) >>
      default: option!(do_parse!(
        cond!(!fields.is_empty(), punct!(,)) >>
        keyword!(default) >>
        punct!(=) >>
        default: syn!(LitInt) >>
        (default)
      )) >>
      option!(punct!(,)) >>
      (Bitfield {
        fields: fields.into_iter().collect(),
        default,
      })
    )) >>
    (parens.1)
  ));
}

impl Synom for Field {
  named!(parse -> Self, do_parse!(
    ident: syn!(Ident) >>
    parens: parens!(do_parse!(
      mode: syn!(Mode) >>
      punct!(,) >>
      offset: syn!(LitInt) >>
      width: option!(do_parse!(
        punct!(,) >>
        width: syn!(LitInt) >>
        (width)
      )) >>
      doc: option!(do_parse!(
        punct!(,) >>
        doc: syn!(LitStr) >>
        (doc)
      )) >>
      (Field { ident, mode, offset, width, doc })
    )) >>
    (parens.1)
  ));
}

impl Synom for Mode {
  named!(parse -> Self, do_parse!(
    ident: syn!(Ident) >>
    mode: switch!(value!(ident.as_ref()),
      "r" => value!(Mode::Read) |
      "rw" => value!(Mode::ReadWrite) |
      "w" => value!(Mode::Write) |
      _ => reject!()
    ) >>
    (mode)
  ));
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
  let (def_site, call_site) = (Span::def_site(), Span::call_site());
  let input = parse::<DeriveInput>(input).unwrap();
  let input_span = input.span();
  let DeriveInput {
    attrs, ident, data, ..
  } = input;
  let scope =
    Ident::from(format!("__bitfield_{}", ident.as_ref().to_snake_case()));
  let var = quote!(self);
  let zero_index = Index {
    index: 0,
    span: call_site,
  };
  let access = quote_spanned!(call_site => #var.#zero_index);
  let bitfield = attrs.into_iter().find(|attr| {
    if_chain! {
      if attr.path.leading_colon.is_none();
      if let Some(Pair::End(x)) = attr.path.segments.first();
      if let PathArguments::None = x.arguments;
      if x.ident == "bitfield";
      then { true } else { false }
    }
  });
  let Bitfield { fields, default } = match bitfield {
    Some(attr) => try_parse2!(attr.span(), attr.tts),
    None => Bitfield::default(),
  };
  let default =
    default.unwrap_or_else(|| LitInt::new(0, IntSuffix::None, def_site));
  let bits = if_chain! {
    if let Data::Struct(x) = data;
    if let Fields::Unnamed(mut x) = x.fields;
    if let Some(Pair::End(x)) = x.unnamed.pop();
    if let Type::Path(x) = x.ty;
    then {
      x
    } else {
      return emit_err(
        input_span,
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
      let width =
        width.unwrap_or_else(|| LitInt::new(1, IntSuffix::None, def_site));
      let mut attrs = vec![quote!(#[inline(always)])];
      if let Some(doc) = doc {
        attrs.push(quote!(#[doc = #doc]));
      }
      let attrs = &attrs;
      if width.value() == 1 {
        if mode.is_read() {
          let read_bit = Ident::new(&format!("{}", ident), call_site);
          fields.push(quote! {
            #(#attrs)*
            pub fn #read_bit(&self) -> bool {
              unsafe { self.read_bit(#offset as #bits) }
            }
          });
        }
        if mode.is_write() {
          let set_bit = Ident::new(&format!("set_{}", ident), call_site);
          let clear_bit = Ident::new(&format!("clear_{}", ident), call_site);
          let toggle_bit = Ident::new(&format!("toggle_{}", ident), call_site);
          fields.push(quote! {
            #(#attrs)*
            pub fn #set_bit(&mut self) -> &mut Self {
              unsafe { self.set_bit(#offset as #bits) };
              self
            }
          });
          fields.push(quote! {
            #(#attrs)*
            pub fn #clear_bit(&mut self) -> &mut Self {
              unsafe { self.clear_bit(#offset as #bits) };
              self
            }
          });
          fields.push(quote! {
            #(#attrs)*
            pub fn #toggle_bit(&mut self) -> &mut Self {
              unsafe { self.toggle_bit(#offset as #bits) };
              self
            }
          });
        }
      } else {
        if mode.is_read() {
          let read_bits = Ident::new(&format!("{}", ident), call_site);
          fields.push(quote! {
            #(#attrs)*
            pub fn #read_bits(&self) -> #bits {
              unsafe { self.read_bits(#offset as #bits, #width as #bits) }
            }
          });
        }
        if mode.is_write() {
          let write_bits = Ident::new(&format!("write_{}", ident), call_site);
          fields.push(quote! {
            #(#attrs)*
            pub fn #write_bits(&mut self, bits: #bits) -> &mut Self {
              unsafe {
                self.write_bits(#offset as #bits, #width as #bits, bits);
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
    mod #scope {
      extern crate drone_core;

      use self::drone_core::bitfield::Bitfield;

      impl Bitfield for #ident {
        type Bits = #bits;

        const DEFAULT: #bits = #default;

        #[inline(always)]
        unsafe fn from_bits(bits: #bits) -> Self {
          #ident(bits)
        }

        #[inline(always)]
        fn bits(&self) -> #bits {
          #access
        }

        #[inline(always)]
        fn bits_mut(&mut self) -> &mut #bits {
          &mut #access
        }
      }

      impl #ident {
        #(#field_tokens)*
      }
    }
  };
  expanded.into()
}
