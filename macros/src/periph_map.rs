use drone_macros_core::{parse_error, parse_ident, unkeywordize, CfgCond, CfgCondExt};
use heck::{ToSnakeCase, ToUpperCamelCase};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream, Result};
use syn::{
    braced, parenthesized, parse_macro_input, token, Attribute, Ident, ImplItem, Path, Token,
};

const MACRO_PREFIX: &str = "periph_";
const TRAIT_SUFFIX: &str = "Map";

struct Input {
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
    variants: Vec<Variant>,
}

#[derive(Default)]
struct Variant {
    ident: Option<Ident>,
    path: Option<Ident>,
    variant: Option<(Ident, Ident)>,
    traits: Vec<Ident>,
    fields: Vec<Field>,
}

struct Field {
    features: CfgCond,
    ident: Ident,
    path: Option<Ident>,
    traits: Vec<Ident>,
}

impl Parse for Input {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let macro_attrs = input.call(Attribute::parse_outer)?;
        input.parse::<Token![pub]>()?;
        input.parse::<Token![macro]>()?;
        let macro_ident = input.parse::<Ident>()?;
        if !macro_ident.to_string().starts_with(MACRO_PREFIX) {
            return Err(
                input.error(format!("Expected an ident which starts with `{MACRO_PREFIX}`"))
            );
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
        parse_ident!(input, struct_ident);
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
        let mut variants = Vec::new();
        while !content.is_empty() {
            variants.push(content.parse()?);
        }
        if variants.is_empty() {
            variants.push(Variant::default());
        }
        Ok(Self { features, ident, variants })
    }
}

impl Parse for Variant {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let ident =
            if input.parse::<Option<Token![@]>>()?.is_some() { Some(input.parse()?) } else { None };
        let mut path = None;
        let mut variant = None;
        let mut traits = Vec::new();
        let mut fields = Vec::new();
        let is_end = || !(input.is_empty() || (ident.is_some() && input.peek(Token![@])));
        if is_end() {
            path = Some(input.parse()?);
            if input.peek(token::Paren) {
                let content;
                parenthesized!(content in input);
                variant = Some((content.parse()?, content.parse()?));
            }
            while !input.peek(Token![;]) {
                traits.push(input.parse()?);
            }
            input.parse::<Token![;]>()?;
            while is_end() {
                fields.push(input.parse()?);
            }
        };
        Ok(Self { ident, path, variant, traits, fields })
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
        Ok(Self { features, ident, path, traits })
    }
}

#[allow(clippy::too_many_lines, clippy::cognitive_complexity)]
pub fn proc_macro(input: TokenStream) -> TokenStream {
    let Input {
        macro_attrs: periph_macro_attrs,
        macro_ident: periph_macro,
        struct_attrs: periph_ty_attrs,
        struct_ident: periph_ty,
        trait_ident: periph_trait,
        items: periph_items,
        root_path,
        macro_root_path,
        blocks,
    } = &parse_macro_input!(input);
    let core_urt = quote!(::drone_core::reg::tag::Urt);
    let core_srt = quote!(::drone_core::reg::tag::Srt);
    let core_crt = quote!(::drone_core::reg::tag::Crt);

    let mut tokens = Vec::new();
    let mut macro_tokens = Vec::new();
    for Block { ident: block_ident, path: block_path, regs } in blocks {
        let block_snk = block_ident.to_string().to_snake_case();
        let block_cml = block_ident.to_string().to_upper_camel_case();
        let block_path = block_path.as_ref().unwrap_or(block_ident);
        let block_path_snk = format_ident!("{}", block_path.to_string().to_snake_case());
        let block_path_ident =
            format_ident!("{}", unkeywordize(block_path_snk.to_string().as_str()));
        for Reg { features: reg_features, ident: reg_ident, variants } in regs {
            let reg_snk = reg_ident.to_string().to_snake_case();
            let reg_cml = reg_ident.to_string().to_upper_camel_case();
            for (variant_i, variant) in variants.iter().enumerate() {
                let Variant { ident: variant_ident, path: var_path, variant, traits, fields } =
                    variant;
                let (var_snk, var_cml) = if let Some(variant_ident) = variant_ident {
                    (
                        format!("{reg_snk}_{}", variant_ident.to_string().to_snake_case()),
                        format!("{reg_cml}{}", variant_ident.to_string().to_upper_camel_case()),
                    )
                } else {
                    (reg_snk.clone(), reg_cml.clone())
                };
                let var_path_snk = var_path
                    .as_ref()
                    .map(|ident| format_ident!("{}", ident.to_string().to_snake_case()));
                let var_path_ident = var_path_snk
                    .as_ref()
                    .map(|ident| format_ident!("{}", unkeywordize(ident.to_string().as_str())));
                let block_var_snk = format_ident!("{}_{}", block_snk, var_snk);
                let block_var_path_snk = var_path_snk
                    .as_ref()
                    .map(|var_path_snk| format_ident!("{}_{}", block_path_snk, var_path_snk));
                let reg_trait = format_ident!("{}{}", block_cml, var_cml);
                let reg_trait_opt = format_ident!("{}{}Opt", block_cml, var_cml);
                let reg_trait_ext = format_ident!("{}{}Ext", block_cml, var_cml);
                let val = format_ident!("{}{}Val", block_cml, var_cml);
                let u_reg = format_ident!("U{}{}", block_cml, var_cml);
                let s_reg = format_ident!("S{}{}", block_cml, var_cml);
                let c_reg = format_ident!("C{}{}", block_cml, var_cml);
                let u_reg_opt = format_ident!("U{}{}Opt", block_cml, var_cml);
                let s_reg_opt = format_ident!("S{}{}Opt", block_cml, var_cml);
                let c_reg_opt = format_ident!("C{}{}Opt", block_cml, var_cml);
                let u_fields = format_ident!("U{}{}Fields", block_cml, var_cml);
                let s_fields = format_ident!("S{}{}Fields", block_cml, var_cml);
                let c_fields = format_ident!("C{}{}Fields", block_cml, var_cml);
                let reg_attrs = reg_features.attrs();
                let (mut reg_shared, mut reg_option) = (false, false);
                for ident in traits {
                    if ident == "Shared" {
                        reg_shared = true;
                    } else if ident == "Option" {
                        reg_option = true;
                    } else {
                        parse_error!("Unknown option `{}`", ident);
                    }
                }
                if reg_shared && reg_option {
                    parse_error!("`Option` and `Shared` can't be used simultaneously");
                }
                if variants.len() > 1 && reg_shared {
                    parse_error!("`Shared` can't be used with multiple variants");
                }
                if reg_option
                    && !variants
                        .iter()
                        .all(|v| v.path.is_none() || v.traits.iter().any(|t| t == "Option"))
                {
                    parse_error!("`Option` should be defined for all variants");
                }
                let reg_root = &quote!(#root_path::#block_path_ident::#var_path_ident);
                let mut reg_fields_tokens = Vec::new();
                let mut fields_tokens = Vec::new();
                let mut u_tokens = Vec::new();
                let mut s_tokens = Vec::new();
                let mut c_tokens = Vec::new();
                for Field {
                    features: field_features,
                    ident: field_ident,
                    path: field_path,
                    traits,
                } in fields
                {
                    let field_path_cml = field_path
                        .as_ref()
                        .map(|ident| format_ident!("{}", ident.to_string().to_upper_camel_case()));
                    let field_path_ident = field_path.as_ref().map(|ident| {
                        format_ident!(
                            "{}",
                            unkeywordize(ident.to_string().to_snake_case().as_str())
                        )
                    });
                    let field_snk = field_ident.to_string().to_snake_case();
                    let field_cml = field_ident.to_string().to_upper_camel_case();
                    let field_ident = format_ident!("{}", unkeywordize(field_snk.clone().as_str()));
                    let block_reg_field_snk =
                        format_ident!("{}_{}_{}", block_snk, var_snk, field_snk);
                    let field_trait = format_ident!("{}{}{}", block_cml, var_cml, field_cml);
                    let field_trait_opt = format_ident!("{}{}{}Opt", block_cml, var_cml, field_cml);
                    let field_trait_ext = format_ident!("{}{}{}Ext", block_cml, var_cml, field_cml);
                    let u_field = format_ident!("U{}{}{}", block_cml, var_cml, field_cml);
                    let s_field = format_ident!("S{}{}{}", block_cml, var_cml, field_cml);
                    let c_field = format_ident!("C{}{}{}", block_cml, var_cml, field_cml);
                    let u_field_opt = format_ident!("U{}{}{}Opt", block_cml, var_cml, field_cml);
                    let s_field_opt = format_ident!("S{}{}{}Opt", block_cml, var_cml, field_cml);
                    let c_field_opt = format_ident!("C{}{}{}Opt", block_cml, var_cml, field_cml);
                    let mut field_option = false;
                    for ident in traits {
                        if ident == "Option" {
                            field_option = true;
                        } else {
                            parse_error!("Unknown option `{}`", ident);
                        }
                    }
                    let mut features = CfgCond::default();
                    features.add_clause(reg_features);
                    features.add_clause(field_features);
                    let field_attrs = features.attrs();
                    let struct_attrs = field_features.attrs();
                    if field_path.is_none() {
                        if reg_shared {
                            tokens.push(quote! {
                                #field_attrs
                                impl #field_trait_opt for #periph_ty {
                                    type #u_field_opt = ();
                                    type #s_field_opt = ();
                                    type #c_field_opt = ();
                                }
                            });
                            macro_tokens.push((features, quote!(#block_reg_field_snk: ())));
                        } else {
                            tokens.push(quote! {
                                #field_attrs
                                impl #field_trait_opt<#periph_ty> for #periph_ty {
                                    type #u_field_opt = ();
                                    type #s_field_opt = ();
                                    type #c_field_opt = ();
                                }
                            });
                        }
                        u_tokens.push(quote! {
                            #struct_attrs
                            #[inline]
                            fn #field_ident(&self) -> &() { &() }
                        });
                        s_tokens.push(quote! {
                            #struct_attrs
                            #[inline]
                            fn #field_ident(&self) -> &() { &() }
                        });
                        c_tokens.push(quote! {
                            #struct_attrs
                            #[inline]
                            fn #field_ident(&self) -> &() { &() }
                        });
                        fields_tokens.push(quote! {
                            #struct_attrs
                            #field_ident: ()
                        });
                    } else if reg_shared {
                        if field_option {
                            tokens.push(quote! {
                                #field_attrs
                                impl #field_trait_opt for #periph_ty {
                                    type #u_field_opt = #reg_root::#field_path_cml<#core_urt>;
                                    type #s_field_opt = #reg_root::#field_path_cml<#core_srt>;
                                    type #c_field_opt = #reg_root::#field_path_cml<#core_crt>;
                                }
                            });
                            tokens.push(quote! {
                                #field_attrs
                                impl #field_trait_ext for #periph_ty {
                                    type #u_field = #reg_root::#field_path_cml<#core_urt>;
                                    type #s_field = #reg_root::#field_path_cml<#core_srt>;
                                    type #c_field = #reg_root::#field_path_cml<#core_crt>;
                                }
                            });
                            tokens.push(quote! {
                                #field_attrs
                                impl #field_trait for #periph_ty {}
                            });
                        } else {
                            tokens.push(quote! {
                                #field_attrs
                                impl #field_trait for #periph_ty {
                                    type #u_field = #reg_root::#field_path_cml<#core_urt>;
                                    type #s_field = #reg_root::#field_path_cml<#core_srt>;
                                    type #c_field = #reg_root::#field_path_cml<#core_crt>;
                                }
                            });
                        }
                        macro_tokens.push((features, quote! {
                            #block_reg_field_snk: $reg.#block_var_path_snk.#field_path_ident
                        }));
                    } else {
                        if field_option {
                            tokens.push(quote! {
                                #field_attrs
                                impl #field_trait_opt<#periph_ty> for #periph_ty {
                                    type #u_field_opt = #reg_root::#field_path_cml<#core_urt>;
                                    type #s_field_opt = #reg_root::#field_path_cml<#core_srt>;
                                    type #c_field_opt = #reg_root::#field_path_cml<#core_crt>;
                                }
                            });
                            tokens.push(quote! {
                                #field_attrs
                                impl #field_trait_ext<#periph_ty> for #periph_ty {
                                    type #u_field = #reg_root::#field_path_cml<#core_urt>;
                                    type #s_field = #reg_root::#field_path_cml<#core_srt>;
                                    type #c_field = #reg_root::#field_path_cml<#core_crt>;
                                }
                            });
                            tokens.push(quote! {
                                #field_attrs
                                impl #field_trait for #periph_ty {}
                            });
                        } else {
                            tokens.push(quote! {
                                #field_attrs
                                impl #field_trait<#periph_ty> for #periph_ty {
                                    type #u_field = #reg_root::#field_path_cml<#core_urt>;
                                    type #s_field = #reg_root::#field_path_cml<#core_srt>;
                                    type #c_field = #reg_root::#field_path_cml<#core_crt>;
                                }
                            });
                        }
                        u_tokens.push(quote! {
                            #struct_attrs
                            #[inline]
                            fn #field_ident(&self) -> &#reg_root::#field_path_cml<#core_urt> {
                                &self.#field_path_ident
                            }
                        });
                        s_tokens.push(quote! {
                            #struct_attrs
                            #[inline]
                            fn #field_ident(&self) -> &#reg_root::#field_path_cml<#core_srt> {
                                &self.#field_path_ident
                            }
                        });
                        c_tokens.push(quote! {
                            #struct_attrs
                            #[inline]
                            fn #field_ident(&self) -> &#reg_root::#field_path_cml<#core_crt> {
                                &self.#field_path_ident
                            }
                        });
                        fields_tokens.push(quote! {
                            #struct_attrs
                            #field_ident: #field_path_ident
                        });
                        reg_fields_tokens.push(quote! {
                            #struct_attrs
                            #field_path_ident
                        });
                    }
                }
                let mut reg_fields_constructor_tokens = reg_fields_tokens.clone();
                let mut reg_fields_destructor_tokens = reg_fields_tokens;
                let mut fields_constructor_tokens = fields_tokens.clone();
                let mut fields_destructor_tokens = fields_tokens;
                if reg_fields_constructor_tokens.is_empty() {
                    reg_fields_constructor_tokens
                        .push(quote!(_marker: ::core::marker::PhantomData));
                    reg_fields_destructor_tokens.push(quote!(_marker));
                    fields_constructor_tokens.push(quote!(_marker: ::core::marker::PhantomData));
                    fields_destructor_tokens.push(quote!(_marker));
                }
                if var_path.is_none() {
                    tokens.push(quote! {
                        #reg_attrs
                        impl #reg_trait_opt for #periph_ty {
                            type #u_reg_opt = ();
                            type #s_reg_opt = ();
                            type #c_reg_opt = ();
                        }
                    });
                    if !reg_shared && variant_i == 0 {
                        macro_tokens.push((reg_features.clone(), quote!(#block_var_snk: ())));
                    }
                } else if reg_shared {
                    if fields.iter().any(|field| field.path.is_some()) {
                        tokens.push(quote! {
                            #reg_attrs
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
                            #reg_attrs
                            impl #reg_trait_opt for #periph_ty {
                                type #u_reg_opt = #reg_root::Reg<#core_urt>;
                                type #s_reg_opt = #reg_root::Reg<#core_srt>;
                                type #c_reg_opt = #reg_root::Reg<#core_crt>;
                            }
                        });
                        tokens.push(quote! {
                            #reg_attrs
                            impl #reg_trait_ext<#periph_ty> for #periph_ty {
                                type #val = #reg_root::Val;
                                type #u_reg = #reg_root::Reg<#core_urt>;
                                type #s_reg = #reg_root::Reg<#core_srt>;
                                type #c_reg = #reg_root::Reg<#core_crt>;
                            }
                        });
                        tokens.push(quote! {
                            #reg_attrs
                            impl #reg_trait for #periph_ty {}
                        });
                    } else {
                        tokens.push(quote! {
                            #reg_attrs
                            impl #reg_trait<#periph_ty> for #periph_ty {
                                type #val = #reg_root::Val;
                                type #u_reg = #reg_root::Reg<#core_urt>;
                                type #s_reg = #reg_root::Reg<#core_srt>;
                                type #c_reg = #reg_root::Reg<#core_crt>;
                            }
                        });
                    }
                    for (variant_j, variant) in variants.iter().enumerate() {
                        if variant_i == variant_j {
                            continue;
                        }
                        let var_snk = variant.ident.as_ref().unwrap().to_string().to_snake_case();
                        let var_path_ident = format_ident!(
                            "{}",
                            unkeywordize(
                                variant.path.as_ref().unwrap().to_string().to_snake_case()
                            )
                        );
                        let var_root = &quote!(#root_path::#block_path_ident::#var_path_ident);
                        let into_variant = format_ident!("into_{}", var_snk);
                        let reg_into_variant =
                            format_ident!("into_{}_{}", block_path_ident, var_path_ident);
                        u_tokens.push(quote! {
                            #[inline]
                            fn #into_variant(self) -> #var_root::Reg<#core_urt> {
                                self.#reg_into_variant()
                            }
                        });
                        s_tokens.push(quote! {
                            #[inline]
                            fn #into_variant(self) -> #var_root::Reg<#core_srt> {
                                self.#reg_into_variant()
                            }
                        });
                        c_tokens.push(quote! {
                            #[inline]
                            fn #into_variant(self) -> #var_root::Reg<#core_crt> {
                                self.#reg_into_variant()
                            }
                        });
                    }
                    tokens.push(quote! {
                        #[allow(clippy::inconsistent_struct_constructor)]
                        #reg_attrs
                        impl #u_reg<#periph_ty> for #reg_root::Reg<#core_urt> {
                            #[inline]
                            fn from_fields(map: #u_fields<#periph_ty>) -> Self {
                                let #u_fields {
                                    #(#fields_destructor_tokens,)*
                                } = map;
                                Self { #(#reg_fields_constructor_tokens),* }
                            }
                            #[inline]
                            fn into_fields(self) -> #u_fields<#periph_ty> {
                                let Self { #(#reg_fields_destructor_tokens),* } = self;
                                #u_fields {
                                    #(#fields_constructor_tokens,)*
                                }
                            }
                            #(#u_tokens)*
                        }
                    });
                    tokens.push(quote! {
                        #[allow(clippy::inconsistent_struct_constructor)]
                        #reg_attrs
                        impl #s_reg<#periph_ty> for #reg_root::Reg<#core_srt> {
                            #[inline]
                            fn from_fields(map: #s_fields<#periph_ty>) -> Self {
                                let #s_fields {
                                    #(#fields_destructor_tokens,)*
                                } = map;
                                Self { #(#reg_fields_constructor_tokens),* }
                            }
                            #[inline]
                            fn into_fields(self) -> #s_fields<#periph_ty> {
                                let Self { #(#reg_fields_destructor_tokens),* } = self;
                                #s_fields {
                                    #(#fields_constructor_tokens,)*
                                }
                            }
                            #(#s_tokens)*
                        }
                    });
                    tokens.push(quote! {
                        #[allow(clippy::inconsistent_struct_constructor)]
                        #reg_attrs
                        impl #c_reg<#periph_ty> for #reg_root::Reg<#core_crt> {
                            #[inline]
                            fn from_fields(map: #c_fields<#periph_ty>) -> Self {
                                let #c_fields {
                                    #(#fields_destructor_tokens,)*
                                } = map;
                                Self { #(#reg_fields_constructor_tokens),* }
                            }
                            #[inline]
                            fn into_fields(self) -> #c_fields<#periph_ty> {
                                let Self { #(#reg_fields_destructor_tokens),* } = self;
                                #c_fields {
                                    #(#fields_constructor_tokens,)*
                                }
                            }
                            #(#c_tokens)*
                        }
                    });
                    if variant_i == 0 {
                        let macro_token =
                            if let Some((from_block_ident, from_var_path_ident)) = variant {
                                let from_variant = format_ident!(
                                    "{}_{}",
                                    from_block_ident.to_string().to_snake_case(),
                                    from_var_path_ident.to_string().to_snake_case()
                                );
                                let into_variant = var_path_snk.as_ref().map(|var_path_snk| {
                                    format_ident!("into_{}_{}", block_path_snk, var_path_snk)
                                });
                                quote!(#block_var_snk: $reg.#from_variant.#into_variant())
                            } else {
                                quote!(#block_var_snk: $reg.#block_var_path_snk)
                            };
                        macro_tokens.push((reg_features.clone(), macro_token));
                    }
                }
            }
        }
    }
    let mut periph_name_cml = periph_trait.to_string();
    periph_name_cml.truncate(periph_name_cml.len() - TRAIT_SUFFIX.len());
    let periph_struct = format_ident!("{}Periph", periph_name_cml);
    for (features, macro_tokens) in macro_tokens.as_slice().transpose() {
        let attrs = features.attrs();
        let macro_root_path = macro_root_path.iter().collect::<Vec<_>>();
        tokens.push(quote! {
            #attrs
            #(#periph_macro_attrs)*
            #[macro_export]
            macro_rules! #periph_macro {
                ($reg:ident) => {
                    $crate #(#macro_root_path)*::#periph_struct::<
                        $crate #(#macro_root_path)*::#periph_ty,
                    > {
                        #(#macro_tokens,)*
                    }
                };
            }
        });
    }

    quote! {
        #(#periph_ty_attrs)*
        pub struct #periph_ty(());

        impl #periph_trait for #periph_ty {
            #(#periph_items)*
        }

        #(#tokens)*
    }
    .into()
}
