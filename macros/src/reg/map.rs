use drone_macros_core::{unkeywordize, NewMod};
use inflector::Inflector;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::Tokens;
use syn::synom::Synom;
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

impl Synom for RegMap {
  named!(parse -> Self, do_parse!(
    block: syn!(NewMod) >>
    regs: many0!(syn!(Reg)) >>
    (RegMap { block, regs })
  ));
}

impl Synom for Reg {
  named!(parse -> Self, do_parse!(
    attrs: many0!(Attribute::parse_outer) >>
    ident: syn!(Ident) >>
    braces: braces!(do_parse!(
      address: syn!(LitInt) >>
      size: map!(syn!(LitInt), |x| x.value() as u8) >>
      reset: syn!(LitInt) >>
      traits: many0!(syn!(Ident)) >>
      punct!(;) >>
      fields: many0!(syn!(Field)) >>
      (Reg { attrs, ident, address, size, reset, traits, fields })
    )) >>
    (braces.1)
  ));
}

impl Synom for Field {
  named!(parse -> Self, do_parse!(
    attrs: many0!(Attribute::parse_outer) >>
    ident: syn!(Ident) >>
    braces: braces!(do_parse!(
      offset: syn!(LitInt) >>
      width: syn!(LitInt) >>
      traits: many0!(syn!(Ident)) >>
      (Field { attrs, ident, offset, width, traits })
    )) >>
    (braces.1)
  ));
}

pub fn proc_macro(input: TokenStream) -> TokenStream {
  let (def_site, call_site) = (Span::def_site(), Span::call_site());
  let RegMap {
    block:
      NewMod {
        attrs: block_attrs,
        vis: block_vis,
        ident: block_ident,
      },
    regs,
  } = try_parse!(call_site, input);
  let mut block_tokens = Vec::new();
  let mut outer_tokens = Vec::new();
  let block_mod =
    gen_block(block_ident, &regs, &mut block_tokens, &mut outer_tokens);

  let expanded = quote_spanned! { def_site =>
    #(#block_attrs)*
    #block_vis mod #block_mod {
      #(#block_tokens)*
    }

    #(#outer_tokens)*
  };
  expanded.into()
}

fn gen_block(
  block_ident: Ident,
  regs: &[Reg],
  block_tokens: &mut Vec<Tokens>,
  outer_tokens: &mut Vec<Tokens>,
) -> Ident {
  let def_site = Span::def_site();
  let block_mod =
    Ident::from(unkeywordize(block_ident.as_ref().to_snake_case().into()));
  let block_prefix = block_ident.as_ref().to_pascal_case();
  let reg = Ident::from("Reg");
  let val = Ident::from("Val");
  let hold = Ident::from("Hold");
  let this = Ident::from("self");
  let new = Ident::from("new");
  for &Reg {
    ref attrs,
    ident,
    ref address,
    size,
    ref reset,
    ref traits,
    ref fields,
  } in regs
  {
    let reg_mod =
      Ident::from(unkeywordize(ident.as_ref().to_snake_case().into()));
    let reg_struct = Ident::from(ident.as_ref().to_pascal_case());
    let reg_alias = Ident::from(format!("{}{}", block_prefix, reg_struct));
    let val_ty = Ident::from(format!("u{}", size));
    let (
      reg_struct_tokens,
      reg_ctor_tokens,
      reg_fork_tokens,
      mut reg_outer_tokens,
    ) = gen_reg(reg, hold, val_ty, fields);
    for trait_ident in traits {
      reg_outer_tokens.push(quote_spanned! { def_site =>
        impl<T: RegTag> #trait_ident<T> for #reg<T> {}
      });
    }
    reg_outer_tokens.push(quote! {
      #(#attrs)*
      #[derive(Bitfield, Clone, Copy)]
      #[bitfield(default = #reset)]
      pub struct Val(#val_ty);
    });
    block_tokens.push(quote_spanned! { def_site =>
      #(#attrs)*
      pub mod #reg_mod {
        extern crate core;
        extern crate drone_core;

        use self::drone_core::reg::prelude::*;
        use self::core::convert::From;
        use self::core::default::Default;

        #(#attrs)*
        #[derive(Clone, Copy)]
        pub struct #reg<T: RegTag> {
          #(#reg_struct_tokens),*
        }

        impl<T: RegTag> #reg<T> {
          #[inline(always)]
          pub(crate) unsafe fn #new() -> Self {
            Self { #(#reg_ctor_tokens,)* }
          }
        }

        impl<T: RegTag> Reg<T> for #reg<T> {
          type Val = #val;

          const ADDRESS: usize = #address;
        }

        impl<'a, T: RegTag + 'a> RegRef<'a, T> for #reg<T> {
          type Hold = #hold<'a, T>;
        }

        impl From<#reg<Urt>> for #reg<Srt> {
          #[inline(always)]
          fn from(_reg: #reg<Urt>) -> Self {
            unsafe { Self::#new() }
          }
        }

        impl From<#reg<Urt>> for #reg<Frt> {
          #[inline(always)]
          fn from(_reg: #reg<Urt>) -> Self {
            unsafe { Self::#new() }
          }
        }

        impl From<#reg<Urt>> for #reg<Crt> {
          #[inline(always)]
          fn from(_reg: #reg<Urt>) -> Self {
            unsafe { Self::#new() }
          }
        }

        impl From<#reg<Srt>> for #reg<Urt> {
          #[inline(always)]
          fn from(_reg: #reg<Srt>) -> Self {
            unsafe { Self::#new() }
          }
        }

        impl From<#reg<Srt>> for #reg<Frt> {
          #[inline(always)]
          fn from(_reg: #reg<Srt>) -> Self {
            unsafe { Self::#new() }
          }
        }

        impl From<#reg<Srt>> for #reg<Crt> {
          #[inline(always)]
          fn from(_reg: #reg<Srt>) -> Self {
            unsafe { Self::#new() }
          }
        }

        impl From<#reg<Frt>> for #reg<Crt> {
          #[inline(always)]
          fn from(_reg: #reg<Frt>) -> Self {
            unsafe { Self::#new() }
          }
        }

        impl RegFork for #reg<Frt> {
          #[inline(always)]
          fn fork(&mut self) -> Self {
            Self { #(#reg_fork_tokens,)* }
          }
        }

        #(#attrs)*
        pub struct #hold<'a, T: RegTag + 'a> {
          reg: &'a #reg<T>,
          val: #val,
        }

        impl<'a, T: RegTag> RegHold<'a, T, #reg<T>> for #hold<'a, T> {
          #[inline(always)]
          unsafe fn new(reg: &'a #reg<T>, val: #val) -> Self {
            Self { reg, val }
          }

          #[inline(always)]
          fn val(&self) -> #val {
            self.val
          }

          #[inline(always)]
          fn set_val(&mut self, val: #val) {
            self.val = val;
          }
        }

        #(#reg_outer_tokens)*
      }

      pub use self::#reg_mod::#reg as #reg_struct;
    });
    outer_tokens.push(quote_spanned! { def_site =>
      pub use #this::#block_mod::#reg_struct as #reg_alias;
    });
  }
  block_mod
}

fn gen_reg(
  reg: Ident,
  hold: Ident,
  val_ty: Ident,
  fields: &[Field],
) -> (Vec<Tokens>, Vec<Tokens>, Vec<Tokens>, Vec<Tokens>) {
  let def_site = Span::def_site();
  let mut reg_struct_tokens = Vec::new();
  let mut reg_ctor_tokens = Vec::new();
  let mut reg_fork_tokens = Vec::new();
  let mut reg_outer_tokens = Vec::new();
  for &Field {
    ref attrs,
    ident,
    ref offset,
    ref width,
    ref traits,
  } in fields
  {
    let suffix = ident.as_ref().to_snake_case();
    let field_struct = Ident::from(ident.as_ref().to_pascal_case());
    let field = Ident::from(unkeywordize(suffix.as_str().into()));
    reg_struct_tokens.push(quote_spanned! { def_site =>
      #(#attrs)*
      pub #field: #field_struct<T>
    });
    reg_ctor_tokens.push(quote_spanned! { def_site =>
      #field: #field_struct(T::default())
    });
    reg_fork_tokens.push(quote_spanned! { def_site =>
      #field: self.#field.fork()
    });
    reg_outer_tokens.push(quote_spanned! { def_site =>
      #(#attrs)*
      #[derive(Clone, Copy)]
      pub struct #field_struct<T: RegTag>(T);

      impl<T: RegTag> RegField<T> for #field_struct<T> {
        type Reg = #reg<T>;

        const OFFSET: usize = #offset;
        const WIDTH: usize = #width;
      }

      impl From<#field_struct<Srt>> for #field_struct<Frt> {
        #[inline(always)]
        fn from(_field: #field_struct<Srt>) -> Self {
          #field_struct(Frt::default())
        }
      }

      impl From<#field_struct<Srt>> for #field_struct<Crt> {
        #[inline(always)]
        fn from(_field: #field_struct<Srt>) -> Self {
          #field_struct(Crt::default())
        }
      }

      impl From<#field_struct<Frt>> for #field_struct<Crt> {
        #[inline(always)]
        fn from(_field: #field_struct<Frt>) -> Self {
          #field_struct(Crt::default())
        }
      }

      impl RegFork for #field_struct<Frt> {
        #[inline(always)]
        fn fork(&mut self) -> Self {
          #field_struct(Frt::default())
        }
      }
    });
    for trait_ident in traits {
      reg_outer_tokens.push(quote_spanned! { def_site =>
        impl<T: RegTag> #trait_ident<T> for #field_struct<T> {}
      });
    }
    if width.value() == 1 {
      reg_outer_tokens.push(quote_spanned! { def_site =>
        impl<T: RegTag> RegFieldBit<T> for #field_struct<T> {}
      });
      if traits.iter().any(|name| name == "RRRegField") {
        reg_outer_tokens.push(quote_spanned! { def_site =>
          impl<'a, T: RegTag> #hold<'a, T> {
            #(#attrs)*
            #[inline(always)]
            pub fn #field(&self) -> bool {
              self.reg.#field.read(&self.val)
            }
          }
        });
      }
      if traits.iter().any(|name| name == "WWRegField") {
        let set_field = Ident::from(format!("set_{}", suffix));
        let clear_field = Ident::from(format!("clear_{}", suffix));
        let toggle_field = Ident::from(format!("toggle_{}", suffix));
        reg_outer_tokens.push(quote_spanned! { def_site =>
          impl<'a, T: RegTag> #hold<'a, T> {
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
      reg_outer_tokens.push(quote_spanned! { def_site =>
        impl<T: RegTag> RegFieldBits<T> for #field_struct<T> {}
      });
      if traits.iter().any(|name| name == "RRRegField") {
        reg_outer_tokens.push(quote_spanned! { def_site =>
          impl<'a, T: RegTag> #hold<'a, T> {
            #(#attrs)*
            #[inline(always)]
            pub fn #field(&self) -> #val_ty {
              self.reg.#field.read(&self.val)
            }
          }
        });
      }
      if traits.iter().any(|name| name == "WWRegField") {
        let write_field = Ident::from(format!("write_{}", suffix));
        reg_outer_tokens.push(quote_spanned! { def_site =>
          impl<'a, T: RegTag> #hold<'a, T> {
            #(#attrs)*
            #[inline(always)]
            pub fn #write_field(&mut self, bits: #val_ty) -> &mut Self {
              self.reg.#field.write(&mut self.val, bits);
              self
            }
          }
        });
      }
    }
  }
  (
    reg_struct_tokens,
    reg_ctor_tokens,
    reg_fork_tokens,
    reg_outer_tokens,
  )
}
