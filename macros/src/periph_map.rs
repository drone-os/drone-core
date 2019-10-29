use drone_macros_core::{compile_error, new_ident, unkeywordize, CfgCond, CfgCondExt};
use inflector::Inflector;
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    braced,
    parse::{Parse, ParseStream, Result},
    parse_macro_input, Attribute, Ident, ImplItem, Path, Token,
};

const MACRO_PREFIX: &str = "periph_";
const TRAIT_SUFFIX: &str = "Map";

struct PeriphMap {
    macro_attrs: Vec<Attribute>,
    macro_ident: Ident,
    struct_attrs: Vec<Attribute>,
    struct_ident: Ident,
    trait_ident: Ident,
    items: Vec<ImplItem>,
    root_path: Path,
    macro_root_path: Option<Path>,
    blocks: Vec<Block>,
}

struct Block {
    ident: Ident,
    path: Option<Ident>,
    regs: Vec<Reg>,
}

struct Reg {
    features: CfgCond,
    ident: Ident,
    path: Option<Ident>,
    traits: Vec<Ident>,
    fields: Vec<Field>,
}

struct Field {
    features: CfgCond,
    ident: Ident,
    path: Option<Ident>,
    traits: Vec<Ident>,
}

impl Parse for PeriphMap {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let macro_attrs = input.call(Attribute::parse_outer)?;
        input.parse::<Token![pub]>()?;
        input.parse::<Token![macro]>()?;
        let macro_ident = input.parse::<Ident>()?;
        if !macro_ident.to_string().starts_with(MACRO_PREFIX) {
            return Err(input.error(format!(
                "Expected an ident which starts with `{}`",
                MACRO_PREFIX
            )));
        }
        input.parse::<Token![;]>()?;
        let struct_attrs = input.call(Attribute::parse_outer)?;
        input.parse::<Token![pub]>()?;
        input.parse::<Token![struct]>()?;
        let struct_ident = input.parse()?;
        input.parse::<Token![;]>()?;
        input.parse::<Token![impl]>()?;
        let trait_ident = input.parse()?;
        input.parse::<Token![for]>()?;
        let ident = input.parse::<Ident>()?;
        if ident != struct_ident {
            return Err(input.error(format!("Should be `{}`", struct_ident)));
        }
        let content;
        braced!(content in input);
        let mut items = Vec::new();
        while !content.is_empty() {
            items.push(content.parse()?);
        }
        let root_path = input.parse()?;
        input.parse::<Token![;]>()?;
        input.parse::<Token![crate]>()?;
        let macro_root_path = if input.peek(Token![;]) {
            input.parse::<Token![;]>()?;
            None
        } else {
            let path = input.parse()?;
            input.parse::<Token![;]>()?;
            Some(path)
        };
        let mut blocks = Vec::new();
        while !input.is_empty() {
            blocks.push(input.parse()?);
        }
        Ok(Self {
            macro_attrs,
            macro_ident,
            struct_attrs,
            struct_ident,
            trait_ident,
            items,
            root_path,
            macro_root_path,
            blocks,
        })
    }
}

impl Parse for Block {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let ident = input.parse()?;
        let content;
        braced!(content in input);
        let path = if content.peek2(Token![;]) {
            let path = content.parse()?;
            content.parse::<Token![;]>()?;
            Some(path)
        } else {
            None
        };
        let mut regs = Vec::new();
        while !content.is_empty() {
            regs.push(content.parse()?);
        }
        Ok(Self { ident, path, regs })
    }
}

impl Parse for Reg {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let features = input.parse()?;
        let ident = input.parse()?;
        let content;
        braced!(content in input);
        let mut traits = Vec::new();
        let mut fields = Vec::new();
        let path = if content.is_empty() {
            None
        } else {
            let path = content.parse()?;
            while !content.peek(Token![;]) {
                traits.push(content.parse()?);
            }
            content.parse::<Token![;]>()?;
            while !content.is_empty() {
                fields.push(content.parse()?);
            }
            Some(path)
        };
        Ok(Self {
            features,
            ident,
            path,
            traits,
            fields,
        })
    }
}

impl Parse for Field {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let features = input.parse()?;
        let ident = input.parse()?;
        let content;
        braced!(content in input);
        let mut traits = Vec::new();
        let path = if content.is_empty() {
            None
        } else {
            let path = content.parse()?;
            while !content.is_empty() {
                traits.push(content.parse()?);
            }
            Some(path)
        };
        Ok(Self {
            features,
            ident,
            path,
            traits,
        })
    }
}

#[allow(clippy::too_many_lines, clippy::cognitive_complexity)]
pub fn proc_macro(input: TokenStream) -> TokenStream {
    let PeriphMap {
        macro_attrs: periph_macro_attrs,
        macro_ident: periph_macro,
        struct_attrs: periph_ty_attrs,
        struct_ident: periph_ty,
        trait_ident: periph_trait,
        items: periph_items,
        root_path,
        macro_root_path,
        blocks,
    } = &parse_macro_input!(input as PeriphMap);
    let core_urt = quote!(::drone_core::reg::tag::Urt);
    let core_srt = quote!(::drone_core::reg::tag::Srt);
    let core_crt = quote!(::drone_core::reg::tag::Crt);

    let mut tokens = Vec::new();
    let mut macro_tokens = Vec::new();
    for Block {
        ident: block_ident,
        path: block_path,
        regs,
    } in blocks
    {
        let block_snk = block_ident.to_string().to_snake_case();
        let block_psc = block_ident.to_string().to_pascal_case();
        let block_path = block_path.as_ref().unwrap_or(block_ident);
        let block_path_snk = new_ident!("{}", block_path.to_string().to_snake_case());
        let block_path_ident = new_ident!("{}", unkeywordize(block_path_snk.to_string().into()));
        for Reg {
            features: reg_features,
            ident: reg_ident,
            path: reg_path,
            traits,
            fields,
        } in regs
        {
            let reg_path_snk = reg_path
                .as_ref()
                .map(|ident| new_ident!("{}", ident.to_string().to_snake_case()));
            let reg_path_ident = reg_path_snk
                .as_ref()
                .map(|ident| new_ident!("{}", unkeywordize(ident.to_string().into())));
            let reg_snk = reg_ident.to_string().to_snake_case();
            let reg_psc = reg_ident.to_string().to_pascal_case();
            let block_reg_snk = new_ident!("{}_{}", block_snk, reg_snk);
            let block_reg_path_snk = reg_path_snk
                .as_ref()
                .map(|reg_path_snk| new_ident!("{}_{}", block_path_snk, reg_path_snk));
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
            let (mut reg_shared, mut reg_option) = (false, false);
            for ident in traits {
                if ident == "Shared" {
                    reg_shared = true;
                } else if ident == "Option" {
                    reg_option = true;
                } else {
                    compile_error!("Unknown option `{}`", ident);
                }
            }
            if reg_shared && reg_option {
                compile_error!("`Option` and `Shared` can't be used simultaneously");
            }
            let reg_root = &quote!(#root_path::#block_path_ident::#reg_path_ident);
            let mut reg_fields_tokens = Vec::new();
            let mut fields_reg_tokens = Vec::new();
            let mut fields_tokens = Vec::new();
            let mut u_methods = Vec::new();
            let mut s_methods = Vec::new();
            let mut c_methods = Vec::new();
            for Field {
                features: field_features,
                ident: field_ident,
                path: field_path,
                traits,
            } in fields
            {
                let field_path_psc = field_path
                    .as_ref()
                    .map(|ident| new_ident!("{}", ident.to_string().to_pascal_case()));
                let field_path_ident = field_path.as_ref().map(|ident| {
                    new_ident!("{}", unkeywordize(ident.to_string().to_snake_case().into()))
                });
                let field_snk = field_ident.to_string().to_snake_case();
                let field_psc = field_ident.to_string().to_pascal_case();
                let field_ident = new_ident!("{}", unkeywordize(field_snk.clone().into()));
                let block_reg_field_snk = new_ident!("{}_{}_{}", block_snk, reg_snk, field_snk);
                let field_trait = new_ident!("{}{}{}", block_psc, reg_psc, field_psc);
                let field_trait_opt = new_ident!("{}{}{}Opt", block_psc, reg_psc, field_psc);
                let field_trait_ext = new_ident!("{}{}{}Ext", block_psc, reg_psc, field_psc);
                let u_field = new_ident!("U{}{}{}", block_psc, reg_psc, field_psc);
                let s_field = new_ident!("S{}{}{}", block_psc, reg_psc, field_psc);
                let c_field = new_ident!("C{}{}{}", block_psc, reg_psc, field_psc);
                let u_field_opt = new_ident!("U{}{}{}Opt", block_psc, reg_psc, field_psc);
                let s_field_opt = new_ident!("S{}{}{}Opt", block_psc, reg_psc, field_psc);
                let c_field_opt = new_ident!("C{}{}{}Opt", block_psc, reg_psc, field_psc);
                let mut field_option = false;
                for ident in traits {
                    if ident == "Option" {
                        field_option = true;
                    } else {
                        compile_error!("Unknown option `{}`", ident);
                    }
                }
                let mut features = CfgCond::default();
                features.add_clause(&reg_features);
                features.add_clause(&field_features);
                let field_attrs = &features.attrs();
                let struct_attrs = &field_features.attrs();
                if field_path.is_none() {
                    if reg_shared {
                        tokens.push(quote! {
                            #(#field_attrs)*
                            impl #field_trait_opt for #periph_ty {
                                type #u_field_opt = ();
                                type #s_field_opt = ();
                                type #c_field_opt = ();
                            }
                        });
                        macro_tokens.push((features, quote!(#block_reg_field_snk: ())));
                    } else {
                        tokens.push(quote! {
                            #(#field_attrs)*
                            impl #field_trait_opt<#periph_ty> for #periph_ty {
                                type #u_field_opt = ();
                                type #s_field_opt = ();
                                type #c_field_opt = ();
                            }
                        });
                    }
                    u_methods.push(quote! {
                        #(#struct_attrs)*
                        #[inline]
                        fn #field_ident(&self) -> &() { &() }
                    });
                    s_methods.push(quote! {
                        #(#struct_attrs)*
                        #[inline]
                        fn #field_ident(&self) -> &() { &() }
                    });
                    c_methods.push(quote! {
                        #(#struct_attrs)*
                        #[inline]
                        fn #field_ident(&self) -> &() { &() }
                    });
                    fields_tokens.push(quote! {
                        #(#struct_attrs)*
                        #field_ident: ()
                    });
                } else if reg_shared {
                    if field_option {
                        tokens.push(quote! {
                            #(#field_attrs)*
                            impl #field_trait_opt for #periph_ty {
                                type #u_field_opt = #reg_root::#field_path_psc<#core_urt>;
                                type #s_field_opt = #reg_root::#field_path_psc<#core_srt>;
                                type #c_field_opt = #reg_root::#field_path_psc<#core_crt>;
                            }
                        });
                        tokens.push(quote! {
                            #(#field_attrs)*
                            impl #field_trait_ext for #periph_ty {
                                type #u_field = #reg_root::#field_path_psc<#core_urt>;
                                type #s_field = #reg_root::#field_path_psc<#core_srt>;
                                type #c_field = #reg_root::#field_path_psc<#core_crt>;
                            }
                        });
                        tokens.push(quote! {
                            #(#field_attrs)*
                            impl #field_trait for #periph_ty {}
                        });
                    } else {
                        tokens.push(quote! {
                            #(#field_attrs)*
                            impl #field_trait for #periph_ty {
                                type #u_field = #reg_root::#field_path_psc<#core_urt>;
                                type #s_field = #reg_root::#field_path_psc<#core_srt>;
                                type #c_field = #reg_root::#field_path_psc<#core_crt>;
                            }
                        });
                    }
                    macro_tokens.push((features, quote! {
                        #block_reg_field_snk: $reg.#block_reg_path_snk.#field_path_ident
                    }));
                } else {
                    if field_option {
                        tokens.push(quote! {
                            #(#field_attrs)*
                            impl #field_trait_opt<#periph_ty> for #periph_ty {
                                type #u_field_opt = #reg_root::#field_path_psc<#core_urt>;
                                type #s_field_opt = #reg_root::#field_path_psc<#core_srt>;
                                type #c_field_opt = #reg_root::#field_path_psc<#core_crt>;
                            }
                        });
                        tokens.push(quote! {
                            #(#field_attrs)*
                            impl #field_trait_ext<#periph_ty> for #periph_ty {
                                type #u_field = #reg_root::#field_path_psc<#core_urt>;
                                type #s_field = #reg_root::#field_path_psc<#core_srt>;
                                type #c_field = #reg_root::#field_path_psc<#core_crt>;
                            }
                        });
                        tokens.push(quote! {
                            #(#field_attrs)*
                            impl #field_trait for #periph_ty {}
                        });
                    } else {
                        tokens.push(quote! {
                            #(#field_attrs)*
                            impl #field_trait<#periph_ty> for #periph_ty {
                                type #u_field = #reg_root::#field_path_psc<#core_urt>;
                                type #s_field = #reg_root::#field_path_psc<#core_srt>;
                                type #c_field = #reg_root::#field_path_psc<#core_crt>;
                            }
                        });
                    }
                    u_methods.push(quote! {
                        #(#struct_attrs)*
                        #[inline]
                        fn #field_ident(&self) -> &#reg_root::#field_path_psc<#core_urt> {
                            &self.#field_path_ident
                        }
                    });
                    s_methods.push(quote! {
                        #(#struct_attrs)*
                        #[inline]
                        fn #field_ident(&self) -> &#reg_root::#field_path_psc<#core_srt> {
                            &self.#field_path_ident
                        }
                    });
                    c_methods.push(quote! {
                        #(#struct_attrs)*
                        #[inline]
                        fn #field_ident(&self) -> &#reg_root::#field_path_psc<#core_crt> {
                            &self.#field_path_ident
                        }
                    });
                    fields_reg_tokens.push(quote! {
                        #(#struct_attrs)*
                        #field_ident: #field_path_ident
                    });
                    reg_fields_tokens.push(quote! {
                        #(#struct_attrs)*
                        #field_path_ident
                    });
                }
            }
            let reg_fields_tokens = &reg_fields_tokens;
            let fields_reg_tokens = &fields_reg_tokens;
            let fields_tokens = &fields_tokens;
            if reg_path.is_none() {
                tokens.push(quote! {
                    #(#reg_attrs)*
                    impl #reg_trait_opt for #periph_ty {
                        type #u_reg_opt = ();
                        type #s_reg_opt = ();
                        type #c_reg_opt = ();
                    }
                });
                if !reg_shared {
                    macro_tokens.push((reg_features.clone(), quote!(#block_reg_snk: ())));
                }
            } else if reg_shared {
                if fields.iter().any(|field| field.path.is_some()) {
                    tokens.push(quote! {
                        #(#reg_attrs)*
                        impl #reg_trait for #periph_ty {
                            type #val = #reg_root::Val;
                            type #u_reg = #reg_root::Reg<#core_urt>;
                            type #s_reg = #reg_root::Reg<#core_srt>;
                            type #c_reg = #reg_root::Reg<#core_crt>;
                        }
                    });
                }
            } else {
                if reg_option {
                    tokens.push(quote! {
                        #(#reg_attrs)*
                        impl #reg_trait_opt for #periph_ty {
                            type #u_reg_opt = #reg_root::Reg<#core_urt>;
                            type #s_reg_opt = #reg_root::Reg<#core_srt>;
                            type #c_reg_opt = #reg_root::Reg<#core_crt>;
                        }
                    });
                    tokens.push(quote! {
                        #(#reg_attrs)*
                        impl #reg_trait_ext<#periph_ty> for #periph_ty {
                            type #val = #reg_root::Val;
                            type #u_reg = #reg_root::Reg<#core_urt>;
                            type #s_reg = #reg_root::Reg<#core_srt>;
                            type #c_reg = #reg_root::Reg<#core_crt>;
                        }
                    });
                    tokens.push(quote! {
                        #(#reg_attrs)*
                        impl #reg_trait for #periph_ty {}
                    });
                } else {
                    tokens.push(quote! {
                        #(#reg_attrs)*
                        impl #reg_trait<#periph_ty> for #periph_ty {
                            type #val = #reg_root::Val;
                            type #u_reg = #reg_root::Reg<#core_urt>;
                            type #s_reg = #reg_root::Reg<#core_srt>;
                            type #c_reg = #reg_root::Reg<#core_crt>;
                        }
                    });
                }
                tokens.push(quote! {
                    #(#reg_attrs)*
                    impl #u_reg<#periph_ty> for #reg_root::Reg<#core_urt> {
                        #[inline]
                        fn from_fields(map: #u_fields<#periph_ty>) -> Self {
                            let #u_fields {
                                #(#fields_reg_tokens,)*
                                #(#fields_tokens,)*
                            } = map;
                            Self { #(#reg_fields_tokens),* }
                        }
                        #[inline]
                        fn into_fields(self) -> #u_fields<#periph_ty> {
                            let Self { #(#reg_fields_tokens),* } = self;
                            #u_fields {
                                #(#fields_reg_tokens,)*
                                #(#fields_tokens,)*
                            }
                        }
                        #(#u_methods)*
                    }
                });
                tokens.push(quote! {
                    #(#reg_attrs)*
                    impl #s_reg<#periph_ty> for #reg_root::Reg<#core_srt> {
                        #[inline]
                        fn from_fields(map: #s_fields<#periph_ty>) -> Self {
                            let #s_fields {
                                #(#fields_reg_tokens,)*
                                #(#fields_tokens,)*
                            } = map;
                            Self { #(#reg_fields_tokens),* }
                        }
                        #[inline]
                        fn into_fields(self) -> #s_fields<#periph_ty> {
                            let Self { #(#reg_fields_tokens),* } = self;
                            #s_fields {
                                #(#fields_reg_tokens,)*
                                #(#fields_tokens,)*
                            }
                        }
                        #(#s_methods)*
                    }
                });
                tokens.push(quote! {
                    #(#reg_attrs)*
                    impl #c_reg<#periph_ty> for #reg_root::Reg<#core_crt> {
                        #[inline]
                        fn from_fields(map: #c_fields<#periph_ty>) -> Self {
                            let #c_fields {
                                #(#fields_reg_tokens,)*
                                #(#fields_tokens,)*
                            } = map;
                            Self { #(#reg_fields_tokens),* }
                        }
                        #[inline]
                        fn into_fields(self) -> #c_fields<#periph_ty> {
                            let Self { #(#reg_fields_tokens),* } = self;
                            #c_fields {
                                #(#fields_reg_tokens,)*
                                #(#fields_tokens,)*
                            }
                        }
                        #(#c_methods)*
                    }
                });
                macro_tokens.push((
                    reg_features.clone(),
                    quote!(#block_reg_snk: $reg.#block_reg_path_snk),
                ));
            }
        }
    }
    let mut periph_name_psc = periph_trait.to_string();
    periph_name_psc.truncate(periph_name_psc.len() - TRAIT_SUFFIX.len());
    let periph_struct = new_ident!("{}Periph", periph_name_psc);
    for (features, macro_tokens) in macro_tokens.as_slice().transpose() {
        let attrs = &features.attrs();
        tokens.push(quote! {
            #(#attrs)*
            #(#periph_macro_attrs)*
            #[macro_export]
            macro_rules! #periph_macro {
                ($reg:ident) => {
                    $crate#(#macro_root_path)*::#periph_struct::<
                        $crate#(#macro_root_path)*::#periph_ty,
                    > {
                        #(#macro_tokens,)*
                    }
                };
            }
        });
    }

    let expanded = quote! {
        #(#periph_ty_attrs)*
        pub struct #periph_ty(());

        impl #periph_trait for #periph_ty {
            #(#periph_items)*
        }

        #(#tokens)*
    };
    expanded.into()
}
