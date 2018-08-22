use drone_macros_core::{unkeywordize, NewMod};
use inflector::Inflector;
use proc_macro2::{Span, TokenStream};
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
  } = try_parse2!(call_site, input);
  let mut block_tokens = Vec::new();
  let mut outer_tokens = Vec::new();
  let block_mod =
    gen_block(&block_ident, &regs, &mut block_tokens, &mut outer_tokens);

  quote_spanned! { def_site =>
    #(#block_attrs)*
    #block_vis mod #block_mod {
      #(#block_tokens)*
    }

    #(#outer_tokens)*
  }
}

fn gen_block(
  block_ident: &Ident,
  regs: &[Reg],
  block_tokens: &mut Vec<TokenStream>,
  outer_tokens: &mut Vec<TokenStream>,
) -> Ident {
  let (def_site, call_site) = (Span::def_site(), Span::call_site());
  let block_mod = Ident::new(
    &unkeywordize(block_ident.to_string().to_snake_case().into()),
    call_site,
  );
  let block_prefix = block_ident.to_string().to_pascal_case();
  let reg = Ident::new("Reg", call_site);
  let val = Ident::new("Val", call_site);
  let hold = Ident::new("Hold", call_site);
  let this = Ident::new("self", call_site);
  let new = Ident::new("new", call_site);
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
    let mut reg_struct_tokens = Vec::new();
    let mut reg_ctor_tokens = Vec::new();
    let mut reg_fork_tokens = Vec::new();
    let mut reg_outer_tokens = Vec::new();
    gen_reg(
      &reg,
      &hold,
      &val_ty,
      fields,
      &mut reg_struct_tokens,
      &mut reg_ctor_tokens,
      &mut reg_fork_tokens,
      &mut reg_outer_tokens,
    );
    for trait_ident in traits {
      let mut trait_ident = trait_ident.clone();
      trait_ident.set_span(def_site);
      reg_outer_tokens.push(quote_spanned! { def_site =>
        impl<T: RegTag> #trait_ident<T> for #reg<T> {}
      });
    }
    reg_outer_tokens.push(quote! {
      #(#attrs)*
      #[derive(Bitfield, Clone, Copy)]
      #[bitfield(default = #reset)]
      pub struct #val(#val_ty);
    });
    block_tokens.push(quote_spanned! { def_site =>
      #(#attrs)*
      pub mod #reg_mod {
        use extern::drone_core::reg::prelude::*;
        use extern::core::convert::From;
        use extern::core::default::Default;

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

#[cfg_attr(feature = "cargo-clippy", allow(too_many_arguments))]
fn gen_reg(
  reg: &Ident,
  hold: &Ident,
  val_ty: &Ident,
  fields: &[Field],
  reg_struct_tokens: &mut Vec<TokenStream>,
  reg_ctor_tokens: &mut Vec<TokenStream>,
  reg_fork_tokens: &mut Vec<TokenStream>,
  reg_outer_tokens: &mut Vec<TokenStream>,
) {
  let (def_site, call_site) = (Span::def_site(), Span::call_site());
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
      let mut trait_ident = trait_ident.clone();
      trait_ident.set_span(def_site);
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
        let set_field = Ident::new(&format!("set_{}", suffix), call_site);
        let clear_field = Ident::new(&format!("clear_{}", suffix), call_site);
        let toggle_field = Ident::new(&format!("toggle_{}", suffix), call_site);
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
        let write_field = Ident::new(&format!("write_{}", suffix), call_site);
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
}
