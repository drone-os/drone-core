use drone_macros_core::{new_ident, unkeywordize};
use inflector::Inflector;
use proc_macro::TokenStream;
use quote::quote;
use std::collections::HashSet;
use syn::{
  braced,
  parse::{Parse, ParseStream, Result},
  parse_macro_input, Attribute, Ident, LitInt, Token, Visibility,
};

struct Reg {
  attrs: Vec<Attribute>,
  vis: Visibility,
  block: Ident,
  ident: Ident,
  address: LitInt,
  size: u8,
  reset: LitInt,
  traits: Vec<Ident>,
  fields: Vec<Field>,
}

struct Field {
  attrs: Vec<Attribute>,
  ident: Ident,
  offset: LitInt,
  width: LitInt,
  traits: Vec<Ident>,
}

impl Parse for Reg {
  fn parse(input: ParseStream<'_>) -> Result<Self> {
    let attrs = input.call(Attribute::parse_outer)?;
    let vis = input.parse()?;
    input.parse::<Token![mod]>()?;
    let block = input.parse()?;
    let ident = input.parse()?;
    input.parse::<Token![;]>()?;
    let address = input.parse()?;
    let size = input.parse::<LitInt>()?.value() as u8;
    let reset = input.parse()?;
    let mut traits = Vec::new();
    while !input.peek(Token![;]) {
      traits.push(input.parse()?);
    }
    input.parse::<Token![;]>()?;
    let mut fields = Vec::new();
    while !input.is_empty() {
      fields.push(input.parse()?);
    }
    Ok(Self {
      attrs,
      vis,
      block,
      ident,
      address,
      size,
      reset,
      traits,
      fields,
    })
  }
}

impl Parse for Field {
  fn parse(input: ParseStream<'_>) -> Result<Self> {
    let attrs = input.call(Attribute::parse_outer)?;
    let ident = input.parse()?;
    let content;
    braced!(content in input);
    let offset = content.parse()?;
    let width = content.parse()?;
    let mut traits = Vec::new();
    while !content.is_empty() {
      traits.push(content.parse()?);
    }
    Ok(Self {
      attrs,
      ident,
      offset,
      width,
      traits,
    })
  }
}

#[allow(clippy::cognitive_complexity)]
pub fn proc_macro(input: TokenStream) -> TokenStream {
  let Reg {
    attrs,
    vis,
    block,
    ident,
    address,
    size,
    reset,
    traits,
    fields,
  } = parse_macro_input!(input as Reg);
  let t = new_ident!("_T");

  let attrs = &attrs;
  let val_ty = new_ident!("u{}", size);
  let mut imports = traits.iter().cloned().collect::<HashSet<_>>();
  let mut tokens = Vec::new();
  let mut struct_tokens = Vec::new();
  let mut ctor_tokens = Vec::new();
  for Field {
    attrs,
    ident,
    offset,
    width,
    traits,
  } in &fields
  {
    let field_snk = ident.to_string().to_snake_case();
    let field_psc = new_ident!("{}", ident.to_string().to_pascal_case());
    let field_ident = new_ident!("{}", unkeywordize(field_snk.as_str().into()));
    imports.extend(traits.iter().cloned());
    struct_tokens.push(quote! {
      #(#attrs)*
      pub #field_ident: #field_psc<#t>
    });
    ctor_tokens.push(quote! {
      #field_ident: ::drone_core::reg::RegField::<#t>::take()
    });
    tokens.push(quote! {
      #(#attrs)*
      #[derive(Clone, Copy)]
      pub struct #field_psc<#t: ::drone_core::reg::RegTag>(#t);

      impl<#t> ::drone_core::reg::RegField<#t> for #field_psc<#t>
      where
        #t: ::drone_core::reg::RegTag,
      {
        type Reg = Reg<#t>;
        type URegField = #field_psc<::drone_core::reg::Urt>;
        type SRegField = #field_psc<::drone_core::reg::Srt>;
        type CRegField = #field_psc<::drone_core::reg::Crt>;

        const OFFSET: usize = #offset;
        const WIDTH: usize = #width;

        #[inline]
        unsafe fn take() -> Self {
          #field_psc(#t::default())
        }
      }
    });
    for ident in traits {
      tokens.push(quote! {
        impl<#t: ::drone_core::reg::RegTag> #ident<#t> for #field_psc<#t> {}
      });
    }
    if width.value() == 1 {
      tokens.push(quote! {
        impl<#t> ::drone_core::reg::RegFieldBit<#t> for #field_psc<#t>
        where
          #t: ::drone_core::reg::RegTag,
        {
        }
      });
      if traits.iter().any(|name| name == "RRRegField") {
        tokens.push(quote! {
          impl<'a, #t: ::drone_core::reg::RegTag> Hold<'a, #t> {
            #(#attrs)*
            #[inline]
            pub fn #field_ident(&self) -> bool {
              ::drone_core::reg::RRRegFieldBit::read(
                &self.reg.#field_ident,
                &self.val,
              )
            }
          }
        });
      }
      if traits.iter().any(|name| name == "WWRegField") {
        let set_field = new_ident!("set_{}", field_snk);
        let clear_field = new_ident!("clear_{}", field_snk);
        let toggle_field = new_ident!("toggle_{}", field_snk);
        tokens.push(quote! {
          impl<'a, #t: ::drone_core::reg::RegTag> Hold<'a, #t> {
            #(#attrs)*
            #[inline]
            pub fn #set_field(&mut self) -> &mut Self {
              ::drone_core::reg::WWRegFieldBit::set(
                &self.reg.#field_ident,
                &mut self.val,
              );
              self
            }

            #(#attrs)*
            #[inline]
            pub fn #clear_field(&mut self) -> &mut Self {
              ::drone_core::reg::WWRegFieldBit::clear(
                &self.reg.#field_ident,
                &mut self.val,
              );
              self
            }

            #(#attrs)*
            #[inline]
            pub fn #toggle_field(&mut self) -> &mut Self {
              ::drone_core::reg::WWRegFieldBit::toggle(
                &self.reg.#field_ident,
                &mut self.val,
              );
              self
            }
          }
        });
      }
    } else {
      tokens.push(quote! {
        impl<#t> ::drone_core::reg::RegFieldBits<#t> for #field_psc<#t>
        where
          #t: ::drone_core::reg::RegTag,
        {
        }
      });
      if traits.iter().any(|name| name == "RRRegField") {
        tokens.push(quote! {
          impl<'a, #t: ::drone_core::reg::RegTag> Hold<'a, #t> {
            #(#attrs)*
            #[inline]
            pub fn #field_ident(&self) -> #val_ty {
              ::drone_core::reg::RRRegFieldBits::read(
                &self.reg.#field_ident,
                &self.val,
              )
            }
          }
        });
      }
      if traits.iter().any(|name| name == "WWRegField") {
        let write_field = new_ident!("write_{}", field_snk);
        tokens.push(quote! {
          impl<'a, #t: ::drone_core::reg::RegTag> Hold<'a, #t> {
            #(#attrs)*
            #[inline]
            pub fn #write_field(&mut self, bits: #val_ty) -> &mut Self {
              ::drone_core::reg::WWRegFieldBits::write(
                &self.reg.#field_ident,
                &mut self.val,
                bits,
              );
              self
            }
          }
        });
      }
    }
  }
  if fields.is_empty() {
    struct_tokens.push(quote!(_marker: ::core::marker::PhantomData<#t>));
    ctor_tokens.push(quote!(_marker: ::core::marker::PhantomData));
  }
  for ident in traits {
    tokens.push(quote! {
      impl<#t: ::drone_core::reg::RegTag> #ident<#t> for Reg<#t> {}
    });
  }
  let reg_full = new_ident!(
    "{}_{}",
    block.to_string().to_snake_case(),
    ident.to_string().to_snake_case()
  );
  let imports = if imports.is_empty() {
    quote!()
  } else {
    quote!(use super::{#(#imports),*};)
  };

  let expanded = quote! {
    #(#attrs)*
    #vis mod #reg_full {
      #imports
      use ::drone_core::bitfield::Bitfield;

      #(#attrs)*
      #[derive(Bitfield, Clone, Copy)]
      #[bitfield(default = #reset)]
      pub struct Val(#val_ty);

      #(#attrs)*
      #[derive(Clone, Copy)]
      pub struct Reg<#t: ::drone_core::reg::RegTag> {
        #(#struct_tokens),*
      }

      impl<#t: ::drone_core::reg::RegTag> ::drone_core::reg::Reg<#t> for Reg<#t>
      {
        type Val = Val;
        type UReg = Reg<::drone_core::reg::Urt>;
        type SReg = Reg<::drone_core::reg::Srt>;
        type CReg = Reg<::drone_core::reg::Crt>;

        const ADDRESS: usize = #address;

        #[inline]
        unsafe fn take() -> Self {
          Self { #(#ctor_tokens,)* }
        }
      }

      impl<'a, #t> ::drone_core::reg::RegRef<'a, #t> for Reg<#t>
      where
        #t: ::drone_core::reg::RegTag + 'a,
      {
        type Hold = Hold<'a, #t>;
      }

      #(#attrs)*
      pub struct Hold<'a, #t: ::drone_core::reg::RegTag + 'a> {
        reg: &'a Reg<#t>,
        val: Val,
      }

      impl<'a, #t> ::drone_core::reg::RegHold<'a, #t, Reg<#t>> for Hold<'a, #t>
      where
        #t: ::drone_core::reg::RegTag,
      {
        #[inline]
        unsafe fn new(reg: &'a Reg<#t>, val: Val) -> Self {
          Self { reg, val }
        }

        #[inline]
        fn val(&self) -> Val {
          self.val
        }

        #[inline]
        fn set_val(&mut self, val: Val) {
          self.val = val;
        }
      }

      #(#tokens)*
    }
  };
  expanded.into()
}
