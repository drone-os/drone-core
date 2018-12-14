use drone_macros_core::{unkeywordize, CfgFeatures, CfgFeaturesExt};
use inflector::Inflector;
use proc_macro::TokenStream;
use syn::parse::{Parse, ParseStream, Result};
use syn::{Attribute, Ident, LitInt, TraitItem};

struct Res {
  attrs: Vec<Attribute>,
  ident: Ident,
  items: Vec<TraitItem>,
  blocks: Vec<Block>,
}

struct Block {
  ident: Ident,
  regs: Vec<Reg>,
}

struct Reg {
  features: CfgFeatures,
  ident: Ident,
  size: u8,
  traits: Vec<Ident>,
  fields: Vec<Field>,
}

struct Field {
  features: CfgFeatures,
  ident: Ident,
  traits: Vec<Ident>,
}

impl Parse for Res {
  fn parse(input: ParseStream) -> Result<Self> {
    let attrs = input.call(Attribute::parse_outer)?;
    input.parse::<Token![pub]>()?;
    input.parse::<Token![trait]>()?;
    let ident = input.parse()?;
    let content;
    braced!(content in input);
    let mut items = Vec::new();
    while !content.is_empty() {
      items.push(content.parse()?);
    }
    let mut blocks = Vec::new();
    while !input.is_empty() {
      blocks.push(input.parse()?);
    }
    Ok(Self {
      attrs,
      ident,
      items,
      blocks,
    })
  }
}

impl Parse for Block {
  fn parse(input: ParseStream) -> Result<Self> {
    let ident = input.parse()?;
    let content;
    braced!(content in input);
    let mut regs = Vec::new();
    while !content.is_empty() {
      regs.push(content.parse()?);
    }
    Ok(Self { ident, regs })
  }
}

impl Parse for Reg {
  fn parse(input: ParseStream) -> Result<Self> {
    let features = input.parse()?;
    let ident = input.parse()?;
    let content;
    braced!(content in input);
    let size = content.parse::<LitInt>()?.value() as u8;
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
      features,
      ident,
      size,
      traits,
      fields,
    })
  }
}

impl Parse for Field {
  fn parse(input: ParseStream) -> Result<Self> {
    let features = input.parse()?;
    let ident = input.parse()?;
    let content;
    braced!(content in input);
    let mut traits = Vec::new();
    while !content.is_empty() {
      traits.push(content.parse()?);
    }
    Ok(Self {
      features,
      ident,
      traits,
    })
  }
}

#[allow(clippy::cyclomatic_complexity)]
pub fn proc_macro(input: TokenStream) -> TokenStream {
  let Res {
    attrs: res_attrs,
    ident: res_trait,
    items: res_items,
    blocks,
  } = &parse_macro_input!(input as Res);

  let mut tokens = Vec::new();
  let mut res_bounds = Vec::new();
  let mut res_fields = Vec::new();
  for Block {
    ident: block_ident,
    regs,
  } in blocks
  {
    let block_snk = block_ident.to_string().to_snake_case();
    let block_psc = block_ident.to_string().to_pascal_case();
    for Reg {
      features: reg_features,
      ident: reg_ident,
      size,
      traits,
      fields,
    } in regs
    {
      let reg_snk = reg_ident.to_string().to_snake_case();
      let block_reg_snk = new_ident!("{}_{}", block_snk, reg_snk);
      let reg_psc = reg_ident.to_string().to_pascal_case();
      let val_ty = new_ident!("u{}", size);
      let reg_trait = new_ident!("{}{}", block_psc, reg_psc);
      let reg_trait_opt = new_ident!("{}{}Opt", block_psc, reg_psc);
      let reg_trait_ext = new_ident!("{}{}Ext", block_psc, reg_psc);
      let val = new_ident!("{}{}Val", block_psc, reg_psc);
      let u_reg = new_ident!("U{}{}", block_psc, reg_psc);
      let s_reg = new_ident!("S{}{}", block_psc, reg_psc);
      let c_reg = new_ident!("C{}{}", block_psc, reg_psc);
      let u_reg_opt = new_ident!("U{}{}Opt", block_psc, reg_psc);
      let s_reg_opt = new_ident!("S{}{}Opt", block_psc, reg_psc);
      let c_reg_opt = new_ident!("C{}{}Opt", block_psc, reg_psc);
      let u_fields = new_ident!("U{}{}Fields", block_psc, reg_psc);
      let s_fields = new_ident!("S{}{}Fields", block_psc, reg_psc);
      let c_fields = new_ident!("C{}{}Fields", block_psc, reg_psc);
      let reg_attrs = &reg_features.attrs();
      let mut u_traits = Vec::new();
      let mut s_traits = Vec::new();
      let mut c_traits = Vec::new();
      let (mut reg_shared, mut reg_option) = (false, false);
      for ident in traits {
        if ident == "Shared" {
          reg_shared = true;
        } else if ident == "Option" {
          reg_option = true;
        } else {
          u_traits.push(new_ident!("U{}", ident));
          s_traits.push(new_ident!("S{}", ident));
          c_traits.push(new_ident!("C{}", ident));
        }
      }
      if reg_shared && reg_option {
        compile_error!("`Option` and `Shared` can't be used simultaneously");
      }
      let mut u_fields_tokens = Vec::new();
      let mut s_fields_tokens = Vec::new();
      let mut c_fields_tokens = Vec::new();
      let mut u_methods = Vec::new();
      let mut s_methods = Vec::new();
      let mut c_methods = Vec::new();
      let mut reg_bounds = Vec::new();
      for Field {
        features: field_features,
        ident: field_ident,
        traits,
      } in fields
      {
        let field_snk = field_ident.to_string().to_snake_case();
        let field_psc = field_ident.to_string().to_pascal_case();
        let field_ident =
          new_ident!("{}", unkeywordize(field_snk.clone().into()));
        let block_reg_field_snk =
          new_ident!("{}_{}_{}", block_snk, reg_snk, field_snk);
        let field_trait = new_ident!("{}{}{}", block_psc, reg_psc, field_psc);
        let field_trait_opt =
          new_ident!("{}{}{}Opt", block_psc, reg_psc, field_psc);
        let field_trait_ext =
          new_ident!("{}{}{}Ext", block_psc, reg_psc, field_psc);
        let u_field = new_ident!("U{}{}{}", block_psc, reg_psc, field_psc);
        let s_field = new_ident!("S{}{}{}", block_psc, reg_psc, field_psc);
        let c_field = new_ident!("C{}{}{}", block_psc, reg_psc, field_psc);
        let u_field_opt =
          new_ident!("U{}{}{}Opt", block_psc, reg_psc, field_psc);
        let s_field_opt =
          new_ident!("S{}{}{}Opt", block_psc, reg_psc, field_psc);
        let c_field_opt =
          new_ident!("C{}{}{}Opt", block_psc, reg_psc, field_psc);
        let mut u_traits = Vec::new();
        let mut s_traits = Vec::new();
        let mut c_traits = Vec::new();
        let mut field_option = false;
        for ident in traits {
          if ident == "Option" {
            field_option = true;
          } else {
            u_traits.push(new_ident!("U{}", ident));
            s_traits.push(new_ident!("S{}", ident));
            c_traits.push(new_ident!("C{}", ident));
          }
        }
        let mut features = CfgFeatures::default();
        features.add_clause(&reg_features);
        features.add_clause(&field_features);
        let field_attrs = &features.attrs();
        let struct_attrs = &field_features.attrs();
        let field_trait_items = quote! {
          type #u_field: ::drone_core::reg::RegField<
            ::drone_core::reg::Urt,
            Reg = Self::#u_reg,
            URegField = Self::#u_field,
            SRegField = Self::#s_field,
            CRegField = Self::#c_field,
          > #(+ #u_traits)*;
          type #s_field: ::drone_core::reg::RegField<
            ::drone_core::reg::Srt,
            Reg = Self::#s_reg,
            URegField = Self::#u_field,
            SRegField = Self::#s_field,
            CRegField = Self::#c_field,
          > #(+ #s_traits)*;
          type #c_field: ::drone_core::reg::RegField<
            ::drone_core::reg::Crt,
            Reg = Self::#c_reg,
            URegField = Self::#u_field,
            SRegField = Self::#s_field,
            CRegField = Self::#c_field,
          > #(+ #c_traits)*;
        };
        if field_option {
          if reg_shared {
            reg_bounds.push((features, quote!(Self: #field_trait_opt)));
            res_fields.push(quote! {
              #(#field_attrs)*
              pub #block_reg_field_snk: T::#s_field_opt,
            });
            tokens.push(quote! {
              #(#field_attrs)*
              #[allow(missing_docs)]
              pub trait #field_trait_opt {
                type #u_field_opt;
                type #s_field_opt;
                type #c_field_opt;
              }
            });
            tokens.push(quote! {
              #(#field_attrs)*
              #[allow(missing_docs)]
              pub trait #field_trait_ext: #reg_trait {
                #(#field_trait_items)*
              }
            });
            tokens.push(quote! {
              #(#field_attrs)*
              #[allow(missing_docs)]
              pub trait #field_trait
              where
                Self: #res_trait,
                Self: #reg_trait,
                Self: #field_trait_ext,
                Self: #field_trait_opt<
                  #u_field_opt = <Self as #field_trait_ext>::#u_field,
                  #s_field_opt = <Self as #field_trait_ext>::#s_field,
                  #c_field_opt = <Self as #field_trait_ext>::#c_field,
                >,
              {
              }
            });
          } else {
            if reg_option {
              reg_bounds.push((features, quote!(Self: #field_trait_opt<Self>)));
              tokens.push(quote! {
                #(#field_attrs)*
                #[allow(missing_docs)]
                pub trait #field_trait_opt<T: #reg_trait>: #reg_trait_ext<T> {
                  type #u_field_opt;
                  type #s_field_opt;
                  type #c_field_opt;
                }
              });
              tokens.push(quote! {
                #(#field_attrs)*
                #[allow(missing_docs)]
                pub trait #field_trait_ext<T: #reg_trait>: #reg_trait_ext<T> {
                  #(#field_trait_items)*
                }
              });
              tokens.push(quote! {
                #(#field_attrs)*
                #[allow(missing_docs)]
                pub trait #field_trait
                where
                  Self: #reg_trait,
                  Self: #field_trait_ext<Self>,
                  Self: #field_trait_opt<
                    Self,
                    #u_field_opt = <Self as #field_trait_ext<Self>>::#u_field,
                    #s_field_opt = <Self as #field_trait_ext<Self>>::#s_field,
                    #c_field_opt = <Self as #field_trait_ext<Self>>::#c_field,
                  >,
                {
                }
              });
            } else {
              reg_bounds.push((features, quote!(Self: #field_trait_opt<Self>)));
              tokens.push(quote! {
                #(#field_attrs)*
                #[allow(missing_docs)]
                pub trait #field_trait_opt<T: #res_trait>: #reg_trait<T> {
                  type #u_field_opt;
                  type #s_field_opt;
                  type #c_field_opt;
                }
              });
              tokens.push(quote! {
                #(#field_attrs)*
                #[allow(missing_docs)]
                pub trait #field_trait_ext<T: #res_trait>: #reg_trait<T> {
                  #(#field_trait_items)*
                }
              });
              tokens.push(quote! {
                #(#field_attrs)*
                #[allow(missing_docs)]
                pub trait #field_trait
                where
                  Self: #res_trait,
                  Self: #reg_trait<Self>,
                  Self: #field_trait_ext<Self>,
                  Self: #field_trait_opt<
                    Self,
                    #u_field_opt = <Self as #field_trait_ext<Self>>::#u_field,
                    #s_field_opt = <Self as #field_trait_ext<Self>>::#s_field,
                    #c_field_opt = <Self as #field_trait_ext<Self>>::#c_field,
                  >,
                {
                }
              });
            }
            u_fields_tokens.push(quote! {
              #(#struct_attrs)*
              pub #field_ident: T::#u_field_opt,
            });
            s_fields_tokens.push(quote! {
              #(#struct_attrs)*
              pub #field_ident: T::#s_field_opt,
            });
            c_fields_tokens.push(quote! {
              #(#struct_attrs)*
              pub #field_ident: T::#c_field_opt,
            });
            u_methods.push(quote! {
              #(#struct_attrs)*
              fn #field_ident(&self) -> &T::#u_field_opt;
            });
            s_methods.push(quote! {
              #(#struct_attrs)*
              fn #field_ident(&self) -> &T::#s_field_opt;
            });
            c_methods.push(quote! {
              #(#struct_attrs)*
              fn #field_ident(&self) -> &T::#c_field_opt;
            });
          }
        } else if reg_shared {
          reg_bounds.push((features, quote!(Self: #field_trait)));
          res_fields.push(quote! {
            #(#field_attrs)*
            pub #block_reg_field_snk: T::#s_field,
          });
          tokens.push(quote! {
            #(#field_attrs)*
            #[allow(missing_docs)]
            pub trait #field_trait: #reg_trait {
              #(#field_trait_items)*
            }
          });
        } else {
          if reg_option {
            reg_bounds.push((features, quote!(Self: #field_trait<Self>)));
            tokens.push(quote! {
              #(#field_attrs)*
              #[allow(missing_docs)]
              pub trait #field_trait<T: #reg_trait>: #reg_trait_ext<T> {
                #(#field_trait_items)*
              }
            });
          } else {
            reg_bounds.push((features, quote!(Self: #field_trait<Self>)));
            tokens.push(quote! {
              #(#field_attrs)*
              #[allow(missing_docs)]
              pub trait #field_trait<T: #res_trait>: #reg_trait<T> {
                #(#field_trait_items)*
              }
            });
          }
          u_fields_tokens.push(quote! {
            #(#struct_attrs)*
            pub #field_ident: T::#u_field,
          });
          s_fields_tokens.push(quote! {
            #(#struct_attrs)*
            pub #field_ident: T::#s_field,
          });
          c_fields_tokens.push(quote! {
            #(#struct_attrs)*
            pub #field_ident: T::#c_field,
          });
          u_methods.push(quote! {
            #(#struct_attrs)*
            fn #field_ident(&self) -> &T::#u_field;
          });
          s_methods.push(quote! {
            #(#struct_attrs)*
            fn #field_ident(&self) -> &T::#s_field;
          });
          c_methods.push(quote! {
            #(#struct_attrs)*
            fn #field_ident(&self) -> &T::#c_field;
          });
        }
      }
      let u_traits = &u_traits;
      let s_traits = &s_traits;
      let c_traits = &c_traits;
      if reg_option {
        res_bounds.push((reg_features.clone(), quote!(Self: #reg_trait_opt)));
        res_fields.push(quote! {
          #(#reg_attrs)*
          pub #block_reg_snk: T::#s_reg_opt,
        });
        tokens.push(quote! {
          #(#reg_attrs)*
          #[allow(missing_docs)]
          pub trait #reg_trait_opt {
            type #u_reg_opt;
            type #s_reg_opt;
            type #c_reg_opt;
          }
        });
        tokens.push(quote! {
          #(#reg_attrs)*
          #[allow(missing_docs)]
          pub trait #reg_trait_ext<T: #reg_trait> {
            type #val: ::drone_core::bitfield::Bitfield<Bits = #val_ty>;
            type #u_reg: #u_reg<
              T,
              Val = Self::#val,
              UReg = Self::#u_reg,
              SReg = Self::#s_reg,
              CReg = Self::#c_reg,
            >;
            type #s_reg: #s_reg<
              T,
              Val = Self::#val,
              UReg = Self::#u_reg,
              SReg = Self::#s_reg,
              CReg = Self::#c_reg,
            >;
            type #c_reg: #c_reg<
              T,
              Val = Self::#val,
              UReg = Self::#u_reg,
              SReg = Self::#s_reg,
              CReg = Self::#c_reg,
            >;
          }
        });
        tokens.push(quote! {
          #(#reg_attrs)*
          #[allow(missing_docs)]
          pub trait #u_reg<T: #reg_trait>: #(#u_traits)+* {
            fn from_fields(map: #u_fields<T>) -> Self;
            fn into_fields(self) -> #u_fields<T>;
            #(#u_methods)*
          }
        });
        tokens.push(quote! {
          #(#reg_attrs)*
          #[allow(missing_docs)]
          pub trait #s_reg<T: #reg_trait>: #(#s_traits)+* {
            fn from_fields(map: #s_fields<T>) -> Self;
            fn into_fields(self) -> #s_fields<T>;
            #(#s_methods)*
          }
        });
        tokens.push(quote! {
          #(#reg_attrs)*
          #[allow(missing_docs)]
          pub trait #c_reg<T: #reg_trait>: #(#c_traits)+* {
            fn from_fields(map: #c_fields<T>) -> Self;
            fn into_fields(self) -> #c_fields<T>;
            #(#c_methods)*
          }
        });
        tokens.push(quote! {
          #(#reg_attrs)*
          #[allow(missing_docs)]
          pub struct #u_fields<T: #reg_trait> {
            #(#u_fields_tokens)*
          }
        });
        tokens.push(quote! {
          #(#reg_attrs)*
          #[allow(missing_docs)]
          pub struct #s_fields<T: #reg_trait> {
            #(#s_fields_tokens)*
          }
        });
        tokens.push(quote! {
          #(#reg_attrs)*
          #[allow(missing_docs)]
          pub struct #c_fields<T: #reg_trait> {
            #(#c_fields_tokens)*
          }
        });
        for (features, bounds) in reg_bounds.as_slice().transpose() {
          let attrs = &features.attrs();
          tokens.push(quote! {
            #(#reg_attrs)*
            #(#attrs)*
            #[allow(missing_docs)]
            pub trait #reg_trait: #res_trait
            where
              Self: #reg_trait_ext<Self>,
              Self: #reg_trait_opt<
                #u_reg_opt = <Self as #reg_trait_ext<Self>>::#u_reg,
                #s_reg_opt = <Self as #reg_trait_ext<Self>>::#s_reg,
                #c_reg_opt = <Self as #reg_trait_ext<Self>>::#c_reg,
              >,
              #(#bounds,)*
            {
            }
          });
        }
      } else if reg_shared {
        res_bounds.append(&mut reg_bounds);
        tokens.push(quote! {
          #(#reg_attrs)*
          #[allow(missing_docs)]
          pub trait #reg_trait {
            type #val: ::drone_core::bitfield::Bitfield<Bits = #val_ty>;
            type #u_reg: ::drone_core::reg::Reg<
              ::drone_core::reg::Urt,
              Val = Self::#val,
              UReg = Self::#u_reg,
              SReg = Self::#s_reg,
              CReg = Self::#c_reg,
            > #(+ #u_traits)*;
            type #s_reg: ::drone_core::reg::Reg<
              ::drone_core::reg::Srt,
              Val = Self::#val,
              UReg = Self::#u_reg,
              SReg = Self::#s_reg,
              CReg = Self::#c_reg,
            > #(+ #s_traits)*;
            type #c_reg: ::drone_core::reg::Reg<
              ::drone_core::reg::Crt,
              Val = Self::#val,
              UReg = Self::#u_reg,
              SReg = Self::#s_reg,
              CReg = Self::#c_reg,
            > #(+ #c_traits)*;
          }
        });
      } else {
        res_bounds.push((reg_features.clone(), quote!(Self: #reg_trait<Self>)));
        res_bounds.append(&mut reg_bounds);
        res_fields.push(quote! {
          #(#reg_attrs)*
          pub #block_reg_snk: T::#s_reg,
        });
        tokens.push(quote! {
          #(#reg_attrs)*
          #[allow(missing_docs)]
          pub trait #reg_trait<T: #res_trait> {
            type #val: ::drone_core::bitfield::Bitfield<Bits = #val_ty>;
            type #u_reg: #u_reg<
              T,
              Val = Self::#val,
              UReg = Self::#u_reg,
              SReg = Self::#s_reg,
              CReg = Self::#c_reg,
            >;
            type #s_reg: #s_reg<
              T,
              Val = Self::#val,
              UReg = Self::#u_reg,
              SReg = Self::#s_reg,
              CReg = Self::#c_reg,
            >;
            type #c_reg: #c_reg<
              T,
              Val = Self::#val,
              UReg = Self::#u_reg,
              SReg = Self::#s_reg,
              CReg = Self::#c_reg,
            >;
          }
        });
        tokens.push(quote! {
          #(#reg_attrs)*
          #[allow(missing_docs)]
          pub trait #u_reg<T: #res_trait>: #(#u_traits)+* {
            fn from_fields(map: #u_fields<T>) -> Self;
            fn into_fields(self) -> #u_fields<T>;
            #(#u_methods)*
          }
        });
        tokens.push(quote! {
          #(#reg_attrs)*
          #[allow(missing_docs)]
          pub trait #s_reg<T: #res_trait>: #(#s_traits)+* {
            fn from_fields(map: #s_fields<T>) -> Self;
            fn into_fields(self) -> #s_fields<T>;
            #(#s_methods)*
          }
        });
        tokens.push(quote! {
          #(#reg_attrs)*
          #[allow(missing_docs)]
          pub trait #c_reg<T: #res_trait>: #(#c_traits)+* {
            fn from_fields(map: #c_fields<T>) -> Self;
            fn into_fields(self) -> #c_fields<T>;
            #(#c_methods)*
          }
        });
        tokens.push(quote! {
          #(#reg_attrs)*
          #[allow(missing_docs)]
          pub struct #u_fields<T: #res_trait> {
            #(#u_fields_tokens)*
          }
        });
        tokens.push(quote! {
          #(#reg_attrs)*
          #[allow(missing_docs)]
          pub struct #s_fields<T: #res_trait> {
            #(#s_fields_tokens)*
          }
        });
        tokens.push(quote! {
          #(#reg_attrs)*
          #[allow(missing_docs)]
          pub struct #c_fields<T: #res_trait> {
            #(#c_fields_tokens)*
          }
        });
      }
    }
  }
  for (features, bounds) in res_bounds.as_slice().transpose() {
    let attrs = &features.attrs();
    tokens.push(quote! {
      #(#res_attrs)*
      #(#attrs)*
      pub trait #res_trait
      where
        Self: Sized,
        #(#bounds,)*
      {
        #(#res_items)*
      }
    });
  }
  let res_struct = new_ident!("{}Res", res_trait);
  tokens.push(quote! {
    #[allow(missing_docs)]
    pub struct #res_struct<T: #res_trait> {
      #(#res_fields)*
    }
  });

  let expanded = quote! {
    #(#tokens)*
  };
  expanded.into()
}
