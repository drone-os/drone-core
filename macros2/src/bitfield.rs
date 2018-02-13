use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{parse, Attribute, Data, DeriveInput, Fields, Ident, Index, Lit,
          Meta, NestedMeta, PathArguments, Type};
use syn::punctuated::Pair;
use syn::spanned::Spanned;

struct BitField {
  name: Ident,
  offset: u64,
  width: Option<u64>,
  doc: Option<String>,
  read: bool,
  write: bool,
}

pub fn derive(input: TokenStream) -> TokenStream {
  let call_site = Span::call_site();
  let input = parse::<DeriveInput>(input).unwrap();
  let name = input.ident;
  let var = quote!(self);
  let zero_index = Index {
    index: 0,
    span: call_site,
  };
  let access = quote_spanned!(call_site => #var.#zero_index);
  let (mut default, mut fields) = (None, Vec::new());
  for attr in &input.attrs {
    if_chain! {
      if attr.path.leading_colon.is_none();
      if let Some(Pair::End(x)) = attr.path.segments.first();
      if let PathArguments::None = x.arguments;
      if x.ident == "bitfield";
      then {
        if let Some(Meta::List(x)) = attr.interpret_meta() {
          for x in x.nested.iter() {
            if let &NestedMeta::Meta(ref x) = x {
              match x {
                &Meta::NameValue(ref x) => {
                  if x.ident == "default" {
                    if default.is_none() {
                      let lit = &x.lit;
                      default = Some(quote!(#lit));
                    } else {
                      duplicate_attr(attr, x.ident);
                    }
                  }
                }
                &Meta::List(ref x) if x.nested.len() > 1 => {
                  let offset;
                  let (mut width, mut doc) = (None, None);
                  let (mut read, mut write) = (false, false);
                  if let NestedMeta::Meta(Meta::Word(x)) = x.nested[0] {
                    if x == "rw" {
                      read = true;
                      write = true;
                    } else if x == "r" {
                      read = true;
                    } else if x == "w" {
                      write = true;
                    } else {
                      invalid_attr(attr);
                      continue;
                    }
                  } else {
                    invalid_attr(attr);
                    continue;
                  }
                  if let NestedMeta::Literal(Lit::Int(ref x)) = x.nested[1] {
                    offset = x.value()
                  } else {
                    invalid_attr(attr);
                    continue;
                  }
                  if x.nested.len() > 2 {
                    match x.nested[2] {
                      NestedMeta::Literal(Lit::Int(ref x)) => {
                        width = Some(x.value());
                      }
                      NestedMeta::Literal(Lit::Str(ref x)) => {
                        doc = Some(x.value());
                      }
                      _ => {
                        invalid_attr(attr);
                        continue;
                      }
                    }
                  }
                  if x.nested.len() > 3 {
                    if_chain! {
                      if x.nested.len() == 4 && doc.is_none();
                      if let NestedMeta::Literal(Lit::Str(ref x)) = x.nested[3];
                      then {
                        doc = Some(x.value());
                      } else {
                        invalid_attr(attr);
                        continue;
                      }
                    }
                  }
                  fields.push(BitField {
                    name: x.ident,
                    offset,
                    width,
                    doc,
                    read,
                    write,
                  });
                }
                _ => {
                  invalid_attr(attr);
                }
              }
            } else {
              invalid_attr(attr);
            }
          }
        } else {
          invalid_attr(attr);
        }
      }
    }
  }
  let default = default.unwrap_or_else(|| quote!(0));
  let bits = if_chain! {
    if let Data::Struct(ref x) = input.data;
    if let Fields::Unnamed(ref x) = x.fields;
    if let Some(Pair::End(x)) = x.unnamed.first();
    if let Type::Path(ref x) = x.ty;
    then {
      x
    } else {
      invalid_struct(&input);
      return TokenStream::empty();
    }
  };

  let field_tokens = fields
    .into_iter()
    .flat_map(|field| {
      let mut fields = Vec::new();
      let BitField {
        name,
        offset,
        width,
        doc,
        read,
        write,
      } = field;
      let width = width.unwrap_or(1);
      let mut attrs = vec![quote!(#[inline(always)])];
      if let Some(doc) = doc {
        attrs.push(quote!(#[doc = #doc]));
      }
      let attrs = &attrs;
      if width == 1 {
        if read {
          let read_bit = Ident::new(&format!("{}", name), call_site);
          fields.push(quote! {
            #(#attrs)*
            pub fn #read_bit(&self) -> bool {
              unsafe { self.read_bit(#offset as #bits) }
            }
          });
        }
        if write {
          let set_bit = Ident::new(&format!("set_{}", name), call_site);
          let clear_bit = Ident::new(&format!("clear_{}", name), call_site);
          let toggle_bit = Ident::new(&format!("toggle_{}", name), call_site);
          fields.push(quote! {
            #(#attrs)*
            pub fn #set_bit(&mut self) {
              unsafe { self.set_bit(#offset as #bits) };
            }
          });
          fields.push(quote! {
            #(#attrs)*
            pub fn #clear_bit(&mut self) {
              unsafe { self.clear_bit(#offset as #bits) };
            }
          });
          fields.push(quote! {
            #(#attrs)*
            pub fn #toggle_bit(&mut self) {
              unsafe { self.toggle_bit(#offset as #bits) };
            }
          });
        }
      } else {
        if read {
          let read_bits = Ident::new(&format!("{}", name), call_site);
          fields.push(quote! {
            #(#attrs)*
            pub fn #read_bits(&self) -> #bits {
              unsafe { self.read_bits(#offset as #bits, #width as #bits) }
            }
          });
        }
        if write {
          let write_bits = Ident::new(&format!("write_{}", name), call_site);
          fields.push(quote! {
            #(#attrs)*
            pub fn #write_bits(&mut self, bits: #bits) {
              unsafe {
                self.write_bits(#offset as #bits, #width as #bits, bits);
              }
            }
          });
        }
      }
      fields
    })
    .collect::<Vec<_>>();

  let expanded = quote! {
    mod scope {
      extern crate drone_core;

      use self::drone_core::bitfield::Bitfield;

      impl Bitfield for #name {
        type Bits = #bits;

        const DEFAULT: #bits = #default;

        #[inline(always)]
        unsafe fn from_bits(bits: #bits) -> Self {
          #name(bits)
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

      impl #name {
        #(#field_tokens)*
      }
    }
  };
  expanded.into()
}

fn invalid_struct(input: &DeriveInput) {
  input
    .span()
    .unstable()
    .error("Bitfield can be derived only from a tuple struct with one field")
    .emit();
}

fn invalid_attr(attr: &Attribute) {
  attr
    .span()
    .unstable()
    .error("Invalid attribute format")
    .emit();
}

fn duplicate_attr(attr: &Attribute, ident: Ident) {
  attr
    .span()
    .unstable()
    .error(format!("Duplicate key `{}`", ident))
    .emit();
}
