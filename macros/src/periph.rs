use drone_macros_core::{parse_error, unkeywordize, CfgCond, CfgCondExt};
use inflector::Inflector;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    braced,
    parse::{Parse, ParseStream, Result},
    parse_macro_input, Attribute, Ident, LitInt, Token, TraitItem,
};

struct Input {
    trait_attrs: Vec<Attribute>,
    trait_ident: Ident,
    trait_items: Vec<TraitItem>,
    struct_attrs: Vec<Attribute>,
    struct_ident: Ident,
    blocks: Vec<Block>,
}

struct Block {
    ident: Ident,
    regs: Vec<Reg>,
}

struct Reg {
    features: CfgCond,
    ident: Ident,
    variants: Vec<Variant>,
}

struct Variant {
    ident: Option<Ident>,
    size: u8,
    traits: Vec<Ident>,
    fields: Vec<Field>,
}

struct Field {
    features: CfgCond,
    ident: Ident,
    traits: Vec<Ident>,
}

impl Parse for Input {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let trait_attrs = input.call(Attribute::parse_outer)?;
        input.parse::<Token![pub]>()?;
        input.parse::<Token![trait]>()?;
        let trait_ident = input.parse()?;
        let content;
        braced!(content in input);
        let mut trait_items = Vec::new();
        while !content.is_empty() {
            trait_items.push(content.parse()?);
        }
        let struct_attrs = input.call(Attribute::parse_outer)?;
        input.parse::<Token![pub]>()?;
        input.parse::<Token![struct]>()?;
        let struct_ident = input.parse()?;
        input.parse::<Token![;]>()?;
        let mut blocks = Vec::new();
        while !input.is_empty() {
            blocks.push(input.parse()?);
        }
        Ok(Self { trait_attrs, trait_ident, trait_items, struct_attrs, struct_ident, blocks })
    }
}

impl Parse for Block {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
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
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let features = input.parse()?;
        let ident = input.parse()?;
        let content;
        braced!(content in input);
        let mut variants = Vec::new();
        loop {
            variants.push(content.parse()?);
            if content.is_empty() {
                break;
            }
        }
        Ok(Self { features, ident, variants })
    }
}

impl Parse for Variant {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let ident =
            if input.parse::<Option<Token![@]>>()?.is_some() { Some(input.parse()?) } else { None };
        let size = input.parse::<LitInt>()?.base10_parse()?;
        let mut traits = Vec::new();
        while !input.peek(Token![;]) {
            traits.push(input.parse()?);
        }
        input.parse::<Token![;]>()?;
        let mut fields = Vec::new();
        while !(input.is_empty() || (ident.is_some() && input.peek(Token![@]))) {
            fields.push(input.parse()?);
        }
        Ok(Self { ident, size, traits, fields })
    }
}

impl Parse for Field {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let features = input.parse()?;
        let ident = input.parse()?;
        let content;
        braced!(content in input);
        let mut traits = Vec::new();
        while !content.is_empty() {
            traits.push(content.parse()?);
        }
        Ok(Self { features, ident, traits })
    }
}

#[allow(clippy::too_many_lines, clippy::cognitive_complexity)]
pub fn proc_macro(input: TokenStream) -> TokenStream {
    let Input { trait_attrs, trait_ident, trait_items, struct_attrs, struct_ident, blocks } =
        &parse_macro_input!(input);
    let mut tokens = Vec::new();
    let mut periph_bounds = Vec::new();
    let mut periph_fields = Vec::new();
    let mut traits_export = Vec::new();
    let marker_bounds = quote! {
        ::core::marker::Sized
            + ::core::marker::Send
            + ::core::marker::Sync
            + 'static
    };
    for Block { ident: block_ident, regs } in blocks {
        let block_snk = block_ident.to_string().to_snake_case();
        let block_psc = block_ident.to_string().to_pascal_case();
        for Reg { features: reg_features, ident: reg_ident, variants } in regs {
            let reg_snk = reg_ident.to_string().to_snake_case();
            let reg_psc = reg_ident.to_string().to_pascal_case();
            for (variant_i, variant) in variants.iter().enumerate() {
                let Variant { ident: variant_ident, size, traits, fields } = variant;
                let (var_snk, var_psc) = if let Some(variant_ident) = variant_ident {
                    (
                        format!("{}_{}", reg_snk, variant_ident.to_string().to_snake_case()),
                        format!("{}{}", reg_psc, variant_ident.to_string().to_pascal_case()),
                    )
                } else {
                    (reg_snk.clone(), reg_psc.clone())
                };
                let block_var_snk = format_ident!("{}_{}", block_snk, var_snk);
                let val_ty = format_ident!("u{}", size);
                let reg_trait = format_ident!("{}{}", block_psc, var_psc);
                let reg_trait_opt = format_ident!("{}{}Opt", block_psc, var_psc);
                let reg_trait_ext = format_ident!("{}{}Ext", block_psc, var_psc);
                let val = format_ident!("{}{}Val", block_psc, var_psc);
                let u_reg = format_ident!("U{}{}", block_psc, var_psc);
                let s_reg = format_ident!("S{}{}", block_psc, var_psc);
                let c_reg = format_ident!("C{}{}", block_psc, var_psc);
                let u_reg_opt = format_ident!("U{}{}Opt", block_psc, var_psc);
                let s_reg_opt = format_ident!("S{}{}Opt", block_psc, var_psc);
                let c_reg_opt = format_ident!("C{}{}Opt", block_psc, var_psc);
                let u_fields = format_ident!("U{}{}Fields", block_psc, var_psc);
                let s_fields = format_ident!("S{}{}Fields", block_psc, var_psc);
                let c_fields = format_ident!("C{}{}Fields", block_psc, var_psc);
                let reg_attrs = reg_features.attrs();
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
                        u_traits.push(format_ident!("U{}", ident));
                        s_traits.push(format_ident!("S{}", ident));
                        c_traits.push(format_ident!("C{}", ident));
                    }
                }
                if reg_shared && reg_option {
                    parse_error!("`Option` and `Shared` can't be used simultaneously");
                }
                if variants.len() > 1 && reg_shared {
                    parse_error!("`Shared` can't be used with multiple variants");
                }
                if reg_option && !variants.iter().all(|v| v.traits.iter().any(|t| t == "Option")) {
                    parse_error!("`Option` should be defined for all variants");
                }
                let mut u_fields_tokens = Vec::new();
                let mut s_fields_tokens = Vec::new();
                let mut c_fields_tokens = Vec::new();
                let mut u_tokens = Vec::new();
                let mut s_tokens = Vec::new();
                let mut c_tokens = Vec::new();
                let mut reg_bounds = Vec::new();
                for Field { features: field_features, ident: field_ident, traits } in fields {
                    let field_snk = field_ident.to_string().to_snake_case();
                    let field_psc = field_ident.to_string().to_pascal_case();
                    let field_ident = format_ident!("{}", unkeywordize(&field_snk));
                    let block_reg_field_snk =
                        format_ident!("{}_{}_{}", block_snk, var_snk, field_snk);
                    let field_trait = format_ident!("{}{}{}", block_psc, var_psc, field_psc);
                    let field_trait_opt = format_ident!("{}{}{}Opt", block_psc, var_psc, field_psc);
                    let field_trait_ext = format_ident!("{}{}{}Ext", block_psc, var_psc, field_psc);
                    let u_field = format_ident!("U{}{}{}", block_psc, var_psc, field_psc);
                    let s_field = format_ident!("S{}{}{}", block_psc, var_psc, field_psc);
                    let c_field = format_ident!("C{}{}{}", block_psc, var_psc, field_psc);
                    let u_field_opt = format_ident!("U{}{}{}Opt", block_psc, var_psc, field_psc);
                    let s_field_opt = format_ident!("S{}{}{}Opt", block_psc, var_psc, field_psc);
                    let c_field_opt = format_ident!("C{}{}{}Opt", block_psc, var_psc, field_psc);
                    let mut u_traits = Vec::new();
                    let mut s_traits = Vec::new();
                    let mut c_traits = Vec::new();
                    let mut field_option = false;
                    for ident in traits {
                        if ident == "Option" {
                            field_option = true;
                        } else {
                            u_traits.push(format_ident!("U{}", ident));
                            s_traits.push(format_ident!("S{}", ident));
                            c_traits.push(format_ident!("C{}", ident));
                        }
                    }
                    let mut features = CfgCond::default();
                    features.add_clause(reg_features);
                    features.add_clause(field_features);
                    let field_attrs = features.attrs();
                    let struct_attrs = field_features.attrs();
                    let field_trait_items = quote! {
                        type #u_field: ::drone_core::reg::field::RegField<
                            ::drone_core::reg::tag::Urt,
                            Reg = Self::#u_reg,
                            URegField = Self::#u_field,
                            SRegField = Self::#s_field,
                            CRegField = Self::#c_field,
                        > #(+ #u_traits)*;
                        type #s_field: ::drone_core::reg::field::RegField<
                            ::drone_core::reg::tag::Srt,
                            Reg = Self::#s_reg,
                            URegField = Self::#u_field,
                            SRegField = Self::#s_field,
                            CRegField = Self::#c_field,
                        > #(+ #s_traits)*;
                        type #c_field: ::drone_core::reg::field::RegField<
                            ::drone_core::reg::tag::Crt,
                            Reg = Self::#c_reg,
                            URegField = Self::#u_field,
                            SRegField = Self::#s_field,
                            CRegField = Self::#c_field,
                        > #(+ #c_traits)*;
                    };
                    traits_export.push((field_attrs.clone(), field_trait.clone()));
                    if field_option {
                        traits_export.push((field_attrs.clone(), field_trait_opt.clone()));
                        traits_export.push((field_attrs.clone(), field_trait_ext.clone()));
                        if reg_shared {
                            reg_bounds.push((features, quote!(Self: #field_trait_opt)));
                            periph_fields.push(quote! {
                                #[allow(missing_docs)]
                                #field_attrs
                                pub #block_reg_field_snk: T::#s_field_opt,
                            });
                            tokens.push(quote! {
                                #field_attrs
                                #[allow(missing_docs)]
                                pub trait #field_trait_opt {
                                    type #u_field_opt: #marker_bounds;
                                    type #s_field_opt: #marker_bounds;
                                    type #c_field_opt: #marker_bounds;
                                }
                            });
                            tokens.push(quote! {
                                #field_attrs
                                #[allow(missing_docs)]
                                pub trait #field_trait_ext: #reg_trait {
                                    #field_trait_items
                                }
                            });
                            tokens.push(quote! {
                                #field_attrs
                                #[allow(missing_docs)]
                                pub trait #field_trait
                                where
                                    Self: #trait_ident,
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
                            reg_bounds.push((features, quote!(Self: #field_trait_opt<Self>)));
                            if reg_option {
                                tokens.push(quote! {
                                    #field_attrs
                                    #[allow(missing_docs)]
                                    pub trait #field_trait_opt<T: #reg_trait>: #reg_trait_ext<T> {
                                        type #u_field_opt: #marker_bounds;
                                        type #s_field_opt: #marker_bounds;
                                        type #c_field_opt: #marker_bounds;
                                    }
                                });
                                tokens.push(quote! {
                                    #field_attrs
                                    #[allow(missing_docs)]
                                    pub trait #field_trait_ext<T: #reg_trait>: #reg_trait_ext<T> {
                                        #field_trait_items
                                    }
                                });
                                tokens.push(quote! {
                                    #field_attrs
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
                                tokens.push(quote! {
                                    #field_attrs
                                    #[allow(missing_docs)]
                                    pub trait #field_trait_opt<T: #trait_ident>: #reg_trait<T> {
                                        type #u_field_opt: #marker_bounds;
                                        type #s_field_opt: #marker_bounds;
                                        type #c_field_opt: #marker_bounds;
                                    }
                                });
                                tokens.push(quote! {
                                    #field_attrs
                                    #[allow(missing_docs)]
                                    pub trait #field_trait_ext<T: #trait_ident>: #reg_trait<T> {
                                        #field_trait_items
                                    }
                                });
                                tokens.push(quote! {
                                    #field_attrs
                                    #[allow(missing_docs)]
                                    pub trait #field_trait
                                    where
                                        Self: #trait_ident,
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
                                #struct_attrs
                                pub #field_ident: T::#u_field_opt,
                            });
                            s_fields_tokens.push(quote! {
                                #struct_attrs
                                pub #field_ident: T::#s_field_opt,
                            });
                            c_fields_tokens.push(quote! {
                                #struct_attrs
                                pub #field_ident: T::#c_field_opt,
                            });
                            u_tokens.push(quote! {
                                #struct_attrs
                                fn #field_ident(&self) -> &T::#u_field_opt;
                            });
                            s_tokens.push(quote! {
                                #struct_attrs
                                fn #field_ident(&self) -> &T::#s_field_opt;
                            });
                            c_tokens.push(quote! {
                                #struct_attrs
                                fn #field_ident(&self) -> &T::#c_field_opt;
                            });
                        }
                    } else if reg_shared {
                        reg_bounds.push((features, quote!(Self: #field_trait)));
                        periph_fields.push(quote! {
                            #[allow(missing_docs)]
                            #field_attrs
                            pub #block_reg_field_snk: T::#s_field,
                        });
                        tokens.push(quote! {
                            #field_attrs
                            #[allow(missing_docs)]
                            pub trait #field_trait: #reg_trait {
                                #field_trait_items
                            }
                        });
                    } else {
                        reg_bounds.push((features, quote!(Self: #field_trait<Self>)));
                        if reg_option {
                            tokens.push(quote! {
                                #field_attrs
                                #[allow(missing_docs)]
                                pub trait #field_trait<T: #reg_trait>: #reg_trait_ext<T> {
                                    #field_trait_items
                                }
                            });
                        } else {
                            tokens.push(quote! {
                                #field_attrs
                                #[allow(missing_docs)]
                                pub trait #field_trait<T: #trait_ident>: #reg_trait<T> {
                                    #field_trait_items
                                }
                            });
                        }
                        u_fields_tokens.push(quote! {
                            #struct_attrs
                            pub #field_ident: T::#u_field,
                        });
                        s_fields_tokens.push(quote! {
                            #struct_attrs
                            pub #field_ident: T::#s_field,
                        });
                        c_fields_tokens.push(quote! {
                            #struct_attrs
                            pub #field_ident: T::#c_field,
                        });
                        u_tokens.push(quote! {
                            #struct_attrs
                            fn #field_ident(&self) -> &T::#u_field;
                        });
                        s_tokens.push(quote! {
                            #struct_attrs
                            fn #field_ident(&self) -> &T::#s_field;
                        });
                        c_tokens.push(quote! {
                            #struct_attrs
                            fn #field_ident(&self) -> &T::#c_field;
                        });
                    }
                }
                let u_traits = &u_traits;
                let s_traits = &s_traits;
                let c_traits = &c_traits;
                traits_export.push((reg_attrs.clone(), reg_trait.clone()));
                if reg_option {
                    traits_export.push((reg_attrs.clone(), reg_trait_opt.clone()));
                    traits_export.push((reg_attrs.clone(), reg_trait_ext.clone()));
                    traits_export.push((reg_attrs.clone(), u_reg.clone()));
                    traits_export.push((reg_attrs.clone(), s_reg.clone()));
                    traits_export.push((reg_attrs.clone(), c_reg.clone()));
                    periph_bounds.push((reg_features.clone(), quote!(Self: #reg_trait_opt)));
                    for (variant_j, variant) in variants.iter().enumerate() {
                        if variant_i == variant_j {
                            continue;
                        }
                        let var_snk = variant.ident.as_ref().unwrap().to_string().to_snake_case();
                        let var_psc = variant.ident.as_ref().unwrap().to_string().to_pascal_case();
                        let into_variant = format_ident!("into_{}", var_snk);
                        let u_variant = format_ident!("U{}{}{}", block_psc, reg_psc, var_psc);
                        let s_variant = format_ident!("S{}{}{}", block_psc, reg_psc, var_psc);
                        let c_variant = format_ident!("C{}{}{}", block_psc, reg_psc, var_psc);
                        let variant = format_ident!("{}{}{}", block_psc, reg_psc, var_psc);
                        u_tokens.push(quote! {
                            fn #into_variant(self) -> T::#u_variant
                            where
                                T: #variant;
                        });
                        s_tokens.push(quote! {
                            fn #into_variant(self) -> T::#s_variant
                            where
                                T: #variant;
                        });
                        c_tokens.push(quote! {
                            fn #into_variant(self) -> T::#c_variant
                            where
                                T: #variant;
                        });
                    }
                    if variant_i == 0 {
                        periph_fields.push(quote! {
                            #[allow(missing_docs)]
                            #reg_attrs
                            pub #block_var_snk: T::#s_reg_opt,
                        });
                    }
                    tokens.push(quote! {
                        #reg_attrs
                        #[allow(missing_docs)]
                        pub trait #reg_trait_opt {
                            type #u_reg_opt: #marker_bounds;
                            type #s_reg_opt: #marker_bounds;
                            type #c_reg_opt: #marker_bounds;
                        }
                    });
                    tokens.push(quote! {
                        #reg_attrs
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
                        #reg_attrs
                        #[allow(missing_docs)]
                        pub trait #u_reg<T: #reg_trait>: #(#u_traits)+* {
                            fn from_fields(map: #u_fields<T>) -> Self;
                            fn into_fields(self) -> #u_fields<T>;
                            #(#u_tokens)*
                        }
                    });
                    tokens.push(quote! {
                        #reg_attrs
                        #[allow(missing_docs)]
                        pub trait #s_reg<T: #reg_trait>: #(#s_traits)+* {
                            fn from_fields(map: #s_fields<T>) -> Self;
                            fn into_fields(self) -> #s_fields<T>;
                            #(#s_tokens)*
                        }
                    });
                    tokens.push(quote! {
                        #reg_attrs
                        #[allow(missing_docs)]
                        pub trait #c_reg<T: #reg_trait>: #(#c_traits)+* {
                            fn from_fields(map: #c_fields<T>) -> Self;
                            fn into_fields(self) -> #c_fields<T>;
                            #(#c_tokens)*
                        }
                    });
                    tokens.push(quote! {
                        #reg_attrs
                        #[allow(missing_docs)]
                        pub struct #u_fields<T: #reg_trait> {
                            #(#u_fields_tokens)*
                        }
                    });
                    tokens.push(quote! {
                        #reg_attrs
                        #[allow(missing_docs)]
                        pub struct #s_fields<T: #reg_trait> {
                            #(#s_fields_tokens)*
                        }
                    });
                    tokens.push(quote! {
                        #reg_attrs
                        #[allow(missing_docs)]
                        pub struct #c_fields<T: #reg_trait> {
                            #(#c_fields_tokens)*
                        }
                    });
                    for (features, bounds) in reg_bounds.as_slice().transpose() {
                        let attrs = features.attrs();
                        tokens.push(quote! {
                            #reg_attrs
                            #attrs
                            #[allow(missing_docs)]
                            pub trait #reg_trait: #trait_ident
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
                    periph_bounds.append(&mut reg_bounds);
                    tokens.push(quote! {
                        #reg_attrs
                        #[allow(missing_docs)]
                        pub trait #reg_trait {
                            type #val: ::drone_core::bitfield::Bitfield<Bits = #val_ty>;
                            type #u_reg: ::drone_core::reg::Reg<
                                ::drone_core::reg::tag::Urt,
                                Val = Self::#val,
                                UReg = Self::#u_reg,
                                SReg = Self::#s_reg,
                                CReg = Self::#c_reg,
                            > #(+ #u_traits)*;
                            type #s_reg: ::drone_core::reg::Reg<
                                ::drone_core::reg::tag::Srt,
                                Val = Self::#val,
                                UReg = Self::#u_reg,
                                SReg = Self::#s_reg,
                                CReg = Self::#c_reg,
                            > #(+ #s_traits)*;
                            type #c_reg: ::drone_core::reg::Reg<
                                ::drone_core::reg::tag::Crt,
                                Val = Self::#val,
                                UReg = Self::#u_reg,
                                SReg = Self::#s_reg,
                                CReg = Self::#c_reg,
                            > #(+ #c_traits)*;
                        }
                    });
                } else {
                    traits_export.push((reg_attrs.clone(), u_reg.clone()));
                    traits_export.push((reg_attrs.clone(), s_reg.clone()));
                    traits_export.push((reg_attrs.clone(), c_reg.clone()));
                    periph_bounds.push((reg_features.clone(), quote!(Self: #reg_trait<Self>)));
                    periph_bounds.append(&mut reg_bounds);
                    if variant_i == 0 {
                        periph_fields.push(quote! {
                            #[allow(missing_docs)]
                            #reg_attrs
                            pub #block_var_snk: T::#s_reg,
                        });
                    }
                    for (variant_j, variant) in variants.iter().enumerate() {
                        if variant_i == variant_j {
                            continue;
                        }
                        let var_snk = variant.ident.as_ref().unwrap().to_string().to_snake_case();
                        let var_psc = variant.ident.as_ref().unwrap().to_string().to_pascal_case();
                        let into_variant = format_ident!("into_{}", var_snk);
                        let u_variant = format_ident!("U{}{}{}", block_psc, reg_psc, var_psc);
                        let s_variant = format_ident!("S{}{}{}", block_psc, reg_psc, var_psc);
                        let c_variant = format_ident!("C{}{}{}", block_psc, reg_psc, var_psc);
                        u_tokens.push(quote! {
                            fn #into_variant(self) -> T::#u_variant;
                        });
                        s_tokens.push(quote! {
                            fn #into_variant(self) -> T::#s_variant;
                        });
                        c_tokens.push(quote! {
                            fn #into_variant(self) -> T::#c_variant;
                        });
                    }
                    tokens.push(quote! {
                        #reg_attrs
                        #[allow(missing_docs)]
                        pub trait #reg_trait<T: #trait_ident> {
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
                        #reg_attrs
                        #[allow(missing_docs)]
                        pub trait #u_reg<T: #trait_ident>: #(#u_traits)+* {
                            fn from_fields(map: #u_fields<T>) -> Self;
                            fn into_fields(self) -> #u_fields<T>;
                            #(#u_tokens)*
                        }
                    });
                    tokens.push(quote! {
                        #reg_attrs
                        #[allow(missing_docs)]
                        pub trait #s_reg<T: #trait_ident>: #(#s_traits)+* {
                            fn from_fields(map: #s_fields<T>) -> Self;
                            fn into_fields(self) -> #s_fields<T>;
                            #(#s_tokens)*
                        }
                    });
                    tokens.push(quote! {
                        #reg_attrs
                        #[allow(missing_docs)]
                        pub trait #c_reg<T: #trait_ident>: #(#c_traits)+* {
                            fn from_fields(map: #c_fields<T>) -> Self;
                            fn into_fields(self) -> #c_fields<T>;
                            #(#c_tokens)*
                        }
                    });
                    tokens.push(quote! {
                        #reg_attrs
                        #[allow(missing_docs)]
                        pub struct #u_fields<T: #trait_ident> {
                            #(#u_fields_tokens)*
                        }
                    });
                    tokens.push(quote! {
                        #reg_attrs
                        #[allow(missing_docs)]
                        pub struct #s_fields<T: #trait_ident> {
                            #(#s_fields_tokens)*
                        }
                    });
                    tokens.push(quote! {
                        #reg_attrs
                        #[allow(missing_docs)]
                        pub struct #c_fields<T: #trait_ident> {
                            #(#c_fields_tokens)*
                        }
                    });
                }
            }
        }
    }
    for (features, bounds) in periph_bounds.as_slice().transpose() {
        let attrs = features.attrs();
        tokens.push(quote! {
            #(#trait_attrs)*
            #attrs
            pub trait #trait_ident
            where
                Self: #marker_bounds,
                #(#bounds,)*
            {
                #(#trait_items)*
            }
        });
    }
    let traits_export = traits_export
        .into_iter()
        .map(|(attrs, ident)| {
            quote! {
                #attrs
                pub use super::#ident as _;
            }
        })
        .collect::<Vec<_>>();
    if periph_fields.is_empty() {
        periph_fields.push(quote! {
            #[allow(missing_docs)]
            pub _marker: ::core::marker::PhantomData<T>,
        });
    }

    let expanded = quote! {
        #(#tokens)*

        #(#struct_attrs)*
        pub struct #struct_ident<T: #trait_ident> {
            #(#periph_fields)*
        }

        #[allow(missing_docs)]
        pub mod traits {
            #(#traits_export)*
        }
    };
    expanded.into()
}
