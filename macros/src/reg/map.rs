use drone_macros_core::{unkeywordize, NewMod};
use inflector::Inflector;
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use std::collections::HashSet;
use syn::parse::{Parse, ParseStream, Result};
use syn::{Attribute, Ident, LitInt};

struct RegMap {
  block: NewMod,
  regs: Vec<Reg>,
}

struct Reg {
  attrs: Vec<Attribute>,
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

impl Parse for RegMap {
  fn parse(input: ParseStream) -> Result<Self> {
    let block = input.parse()?;
    let mut regs = Vec::new();
    while !input.is_empty() {
      regs.push(input.parse()?);
    }
    Ok(Self { block, regs })
  }
}

impl Parse for Reg {
  fn parse(input: ParseStream) -> Result<Self> {
    let attrs = input.call(Attribute::parse_outer)?;
    let ident = input.parse()?;
    let content;
    braced!(content in input);
    let address = content.parse()?;
    let size = content.parse::<LitInt>()?.value() as u8;
    let reset = content.parse()?;
    let mut traits = Vec::new();
    while !content.peek(Token![;]) {
      traits.push(content.parse()?);
    }
    content.parse::<Token![;]>()?;
    let mut fields = Vec::new();
    while !content.is_empty() {
      fields.push(content.parse()?);
    }
    Ok(Self {
      attrs,
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
  fn parse(input: ParseStream) -> Result<Self> {
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

pub fn proc_macro(input: TokenStream) -> TokenStream {
  let RegMap {
    block:
      NewMod {
        attrs: block_attrs,
        vis: block_vis,
        ident: block_ident,
      },
    regs,
  } = parse_macro_input!(input as RegMap);
  let mut block_tokens = Vec::new();
  let mut outer_tokens = Vec::new();
  let block_mod =
    gen_block(&block_ident, &regs, &mut block_tokens, &mut outer_tokens);

  let expanded = quote! {
    #(#block_attrs)*
    #block_vis mod #block_mod {
      #(#block_tokens)*
    }

    #(#outer_tokens)*
  };
  expanded.into()
}

fn gen_block(
  block_ident: &Ident,
  regs: &[Reg],
  block_tokens: &mut Vec<TokenStream2>,
  outer_tokens: &mut Vec<TokenStream2>,
) -> Ident {
  let (def_site, call_site) = (Span::def_site(), Span::call_site());
  let block_mod = Ident::new(
    &unkeywordize(block_ident.to_string().to_snake_case().into()),
    call_site,
  );
  let block_prefix = block_ident.to_string().to_pascal_case();
  let rt = Ident::new("__rt", def_site);
  let t = Ident::new("T", def_site);
  for &Reg {
    ref attrs,
    ref ident,
    ref address,
    size,
    ref reset,
    ref traits,
    ref fields,
  } in regs
  {
    let reg_mod = Ident::new(
      &unkeywordize(ident.to_string().to_snake_case().into()),
      call_site,
    );
    let reg_struct = Ident::new(&ident.to_string().to_pascal_case(), call_site);
    let reg_alias =
      Ident::new(&format!("{}{}", block_prefix, reg_struct), call_site);
    let val_ty = Ident::new(&format!("u{}", size), call_site);
    let mut imports = traits.iter().cloned().collect();
    let mut reg_struct_tokens = Vec::new();
    let mut reg_ctor_tokens = Vec::new();
    let mut reg_fork_tokens = Vec::new();
    let mut reg_outer_tokens = Vec::new();
    gen_reg(
      &val_ty,
      fields,
      &mut imports,
      &mut reg_struct_tokens,
      &mut reg_ctor_tokens,
      &mut reg_fork_tokens,
      &mut reg_outer_tokens,
    );
    let imports = if imports.is_empty() {
      quote!()
    } else {
      quote!(use super::super::{#(#imports),*};)
    };
    reg_outer_tokens.push(quote! {
      #imports
      mod #rt {
        extern crate core;
        extern crate drone_core;

        pub use self::drone_core::reg::prelude::*;
        pub use self::core::convert::From;
        pub use self::core::default::Default;
        pub use self::core::marker::PhantomData;
      }
    });
    for trait_ident in traits {
      reg_outer_tokens.push(quote! {
        impl<#t: #rt::RegTag> #trait_ident<#t> for Reg<#t> {}
      });
    }
    reg_outer_tokens.push(quote! {
      #(#attrs)*
      #[derive(Bitfield, Clone, Copy)]
      #[bitfield(default = #reset)]
      pub struct Val(#val_ty);
    });
    reg_outer_tokens.push(quote! {
      #(#attrs)*
      #[derive(Clone, Copy)]
      pub struct Reg<#t: #rt::RegTag> {
        #(#reg_struct_tokens),*
      }

      impl<#t: #rt::RegTag> #rt::Reg<#t> for Reg<#t> {
        type Val = Val;

        const ADDRESS: usize = #address;

        #[inline(always)]
        unsafe fn new() -> Self {
          Self { #(#reg_ctor_tokens,)* }
        }
      }

      impl<'a, #t: #rt::RegTag + 'a> #rt::RegRef<'a, #t> for Reg<#t> {
        type Hold = Hold<'a, #t>;
      }

      impl #rt::From<Reg<#rt::Urt>> for Reg<#rt::Srt> {
        #[inline(always)]
        fn from(_reg: Reg<#rt::Urt>) -> Self {
          unsafe { #rt::Reg::new() }
        }
      }

      impl #rt::From<Reg<#rt::Urt>> for Reg<#rt::Frt> {
        #[inline(always)]
        fn from(_reg: Reg<#rt::Urt>) -> Self {
          unsafe { #rt::Reg::new() }
        }
      }

      impl From<Reg<#rt::Urt>> for Reg<#rt::Crt> {
        #[inline(always)]
        fn from(_reg: Reg<#rt::Urt>) -> Self {
          unsafe { #rt::Reg::new() }
        }
      }

      impl From<Reg<#rt::Srt>> for Reg<#rt::Urt> {
        #[inline(always)]
        fn from(_reg: Reg<#rt::Srt>) -> Self {
          unsafe { #rt::Reg::new() }
        }
      }

      impl From<Reg<#rt::Srt>> for Reg<#rt::Frt> {
        #[inline(always)]
        fn from(_reg: Reg<#rt::Srt>) -> Self {
          unsafe { #rt::Reg::new() }
        }
      }

      impl From<Reg<#rt::Srt>> for Reg<#rt::Crt> {
        #[inline(always)]
        fn from(_reg: Reg<#rt::Srt>) -> Self {
          unsafe { #rt::Reg::new() }
        }
      }

      impl From<Reg<#rt::Frt>> for Reg<#rt::Crt> {
        #[inline(always)]
        fn from(_reg: Reg<#rt::Frt>) -> Self {
          unsafe { #rt::Reg::new() }
        }
      }

      impl #rt::RegFork for Reg<#rt::Frt> {
        #[inline(always)]
        fn fork(&mut self) -> Self {
          Self { #(#reg_fork_tokens,)* }
        }
      }

      #(#attrs)*
      pub struct Hold<'a, #t: #rt::RegTag + 'a> {
        reg: &'a Reg<#t>,
        val: Val,
      }

      impl<'a, #t: #rt::RegTag> #rt::RegHold<'a, #t, Reg<#t>> for Hold<'a, #t> {
        #[inline(always)]
        unsafe fn new(reg: &'a Reg<#t>, val: Val) -> Self {
          Self { reg, val }
        }

        #[inline(always)]
        fn val(&self) -> Val {
          self.val
        }

        #[inline(always)]
        fn set_val(&mut self, val: Val) {
          self.val = val;
        }
      }
    });
    block_tokens.push(quote! {
      #(#attrs)*
      pub mod #reg_mod {
        #(#reg_outer_tokens)*
      }

      pub use self::#reg_mod::Reg as #reg_struct;
    });
    outer_tokens.push(quote! {
      pub use self::#block_mod::#reg_struct as #reg_alias;
    });
  }
  block_mod
}

fn gen_reg(
  val_ty: &Ident,
  fields: &[Field],
  imports: &mut HashSet<Ident>,
  reg_struct_tokens: &mut Vec<TokenStream2>,
  reg_ctor_tokens: &mut Vec<TokenStream2>,
  reg_fork_tokens: &mut Vec<TokenStream2>,
  reg_outer_tokens: &mut Vec<TokenStream2>,
) {
  let (def_site, call_site) = (Span::def_site(), Span::call_site());
  let rt = Ident::new("__rt", def_site);
  let t = Ident::new("T", def_site);
  for &Field {
    ref attrs,
    ref ident,
    ref offset,
    ref width,
    ref traits,
  } in fields
  {
    let suffix = ident.to_string().to_snake_case();
    let field_struct =
      Ident::new(&ident.to_string().to_pascal_case(), call_site);
    let field = Ident::new(&unkeywordize(suffix.as_str().into()), call_site);
    imports.extend(traits.iter().cloned());
    reg_struct_tokens.push(quote! {
      #(#attrs)*
      pub #field: #field_struct<#t>
    });
    reg_ctor_tokens.push(quote! {
      #field: #field_struct(#t::default())
    });
    reg_fork_tokens.push(quote! {
      #field: self.#field.fork()
    });
    reg_outer_tokens.push(quote! {
      #(#attrs)*
      #[derive(Clone, Copy)]
      pub struct #field_struct<#t: #rt::RegTag>(#t);

      impl<#t: #rt::RegTag> #rt::RegField<#t> for #field_struct<#t> {
        type Reg = Reg<#t>;

        const OFFSET: usize = #offset;
        const WIDTH: usize = #width;
      }

      impl From<#field_struct<#rt::Srt>> for #field_struct<#rt::Frt> {
        #[inline(always)]
        fn from(_field: #field_struct<#rt::Srt>) -> Self {
          #field_struct(#rt::Frt::default())
        }
      }

      impl From<#field_struct<#rt::Srt>> for #field_struct<#rt::Crt> {
        #[inline(always)]
        fn from(_field: #field_struct<#rt::Srt>) -> Self {
          #field_struct(#rt::Crt::default())
        }
      }

      impl From<#field_struct<#rt::Frt>> for #field_struct<#rt::Crt> {
        #[inline(always)]
        fn from(_field: #field_struct<#rt::Frt>) -> Self {
          #field_struct(#rt::Crt::default())
        }
      }

      impl #rt::RegFork for #field_struct<#rt::Frt> {
        #[inline(always)]
        fn fork(&mut self) -> Self {
          #field_struct(#rt::Frt::default())
        }
      }
    });
    for trait_ident in traits {
      reg_outer_tokens.push(quote! {
        impl<#t: #rt::RegTag> #trait_ident<#t> for #field_struct<#t> {}
      });
    }
    if width.value() == 1 {
      reg_outer_tokens.push(quote! {
        impl<#t: #rt::RegTag> #rt::RegFieldBit<#t> for #field_struct<#t> {}
      });
      if traits.iter().any(|name| name == "RRRegField") {
        reg_outer_tokens.push(quote! {
          impl<'a, #t: #rt::RegTag> Hold<'a, #t> {
            #(#attrs)*
            #[inline(always)]
            pub fn #field(&self) -> bool {
              #rt::RRRegFieldBit::read(&self.reg.#field, &self.val)
            }
          }
        });
      }
      if traits.iter().any(|name| name == "WWRegField") {
        let set_field = Ident::new(&format!("set_{}", suffix), call_site);
        let clear_field = Ident::new(&format!("clear_{}", suffix), call_site);
        let toggle_field = Ident::new(&format!("toggle_{}", suffix), call_site);
        reg_outer_tokens.push(quote! {
          impl<'a, #t: #rt::RegTag> Hold<'a, #t> {
            #(#attrs)*
            #[inline(always)]
            pub fn #set_field(&mut self) -> &mut Self {
              #rt::WWRegFieldBit::set(&self.reg.#field, &mut self.val);
              self
            }

            #(#attrs)*
            #[inline(always)]
            pub fn #clear_field(&mut self) -> &mut Self {
              #rt::WWRegFieldBit::clear(&self.reg.#field, &mut self.val);
              self
            }

            #(#attrs)*
            #[inline(always)]
            pub fn #toggle_field(&mut self) -> &mut Self {
              #rt::WWRegFieldBit::toggle(&self.reg.#field, &mut self.val);
              self
            }
          }
        });
      }
    } else {
      reg_outer_tokens.push(quote! {
        impl<#t: #rt::RegTag> #rt::RegFieldBits<#t> for #field_struct<#t> {}
      });
      if traits.iter().any(|name| name == "RRRegField") {
        reg_outer_tokens.push(quote! {
          impl<'a, #t: #rt::RegTag> Hold<'a, #t> {
            #(#attrs)*
            #[inline(always)]
            pub fn #field(&self) -> #val_ty {
              #rt::RRRegFieldBits::read(&self.reg.#field, &self.val)
            }
          }
        });
      }
      if traits.iter().any(|name| name == "WWRegField") {
        let write_field = Ident::new(&format!("write_{}", suffix), call_site);
        reg_outer_tokens.push(quote! {
          impl<'a, #t: #rt::RegTag> Hold<'a, #t> {
            #(#attrs)*
            #[inline(always)]
            pub fn #write_field(&mut self, bits: #val_ty) -> &mut Self {
              #rt::WWRegFieldBits::write(
                &self.reg.#field,
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
    reg_struct_tokens.push(quote! {
      _marker: #rt::PhantomData<#t>
    });
    reg_ctor_tokens.push(quote! {
      _marker: #rt::PhantomData
    });
    reg_fork_tokens.push(quote! {
      _marker: #rt::PhantomData
    });
  }
}
