use drone_macros_core::unkeywordize;
use inflector::Inflector;
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use std::collections::HashSet;
use syn::{
    braced,
    parse::{Parse, ParseStream, Result},
    parse_macro_input, Attribute, Ident, LitInt, LitStr, Token, Visibility,
};

struct Input {
    variants: Vec<Variant>,
}

struct Variant {
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

impl Parse for Input {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut variants = Vec::new();
        while !input.is_empty() {
            variants.push(input.parse()?);
            if !input.is_empty() {
                input.parse::<Token![;]>()?;
            }
        }
        Ok(Self { variants })
    }
}

impl Parse for Variant {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let vis = input.parse()?;
        let block = input.parse()?;
        let ident = input.parse()?;
        input.parse::<Token![=>]>()?;
        let input2;
        braced!(input2 in input);
        let mut address = None;
        let mut size = None;
        let mut reset = None;
        let mut traits = Vec::new();
        let mut fields = Vec::new();
        while !input2.is_empty() {
            let ident = input2.parse::<Ident>()?;
            input2.parse::<Token![=>]>()?;
            if ident == "address" {
                if address.is_none() {
                    address = Some(input2.parse()?);
                } else {
                    return Err(input2.error("multiple `address` specifications"));
                }
            } else if ident == "size" {
                if size.is_none() {
                    size = Some(input2.parse::<LitInt>()?.base10_parse()?);
                } else {
                    return Err(input2.error("multiple `size` specifications"));
                }
            } else if ident == "reset" {
                if reset.is_none() {
                    reset = Some(input2.parse()?);
                } else {
                    return Err(input2.error("multiple `reset` specifications"));
                }
            } else if ident == "traits" {
                traits.extend(parse_traits(&input2)?);
            } else if ident == "fields" {
                fields.extend(Field::parse_list(&input2)?);
            } else {
                return Err(input2.error(format!("unknown key: `{}`", ident)));
            }
            if !input2.is_empty() {
                input2.parse::<Token![;]>()?;
            }
        }
        Ok(Self {
            attrs,
            vis,
            block,
            ident,
            address: address.ok_or_else(|| input2.error("missing `address` specification"))?,
            size: size.ok_or_else(|| input2.error("missing `size` specification"))?,
            reset: reset.ok_or_else(|| input2.error("missing `reset` specification"))?,
            traits,
            fields,
        })
    }
}

impl Field {
    fn parse_list(input: ParseStream<'_>) -> Result<Vec<Self>> {
        let mut fields = Vec::new();
        let input2;
        braced!(input2 in input);
        while !input2.is_empty() {
            fields.push(input2.parse()?);
            if !input2.is_empty() {
                input2.parse::<Token![;]>()?;
            }
        }
        Ok(fields)
    }
}

impl Parse for Field {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let ident = input.parse()?;
        input.parse::<Token![=>]>()?;
        let input2;
        braced!(input2 in input);
        let mut offset = None;
        let mut width = None;
        let mut traits = Vec::new();
        while !input2.is_empty() {
            let ident = input2.parse::<Ident>()?;
            input2.parse::<Token![=>]>()?;
            if ident == "offset" {
                if offset.is_none() {
                    offset = Some(input2.parse()?);
                } else {
                    return Err(input2.error("multiple `offset` specifications"));
                }
            } else if ident == "width" {
                if width.is_none() {
                    width = Some(input2.parse()?);
                } else {
                    return Err(input2.error("multiple `width` specifications"));
                }
            } else if ident == "traits" {
                traits.extend(parse_traits(&input2)?);
            } else {
                return Err(input2.error(format!("unknown key: `{}`", ident)));
            }
            if !input2.is_empty() {
                input2.parse::<Token![;]>()?;
            }
        }
        Ok(Self {
            attrs,
            ident,
            offset: offset.ok_or_else(|| input2.error("missing `offset` specification"))?,
            width: width.ok_or_else(|| input2.error("missing `width` specification"))?,
            traits,
        })
    }
}

impl Variant {
    #[allow(clippy::too_many_lines, clippy::cognitive_complexity)]
    fn generate(&self) -> TokenStream2 {
        let t = format_ident!("_T");
        let val_ty = format_ident!("u{}", self.size);
        let mut imports = self.traits.iter().cloned().collect::<HashSet<_>>();
        let mut tokens = Vec::new();
        let mut struct_tokens = Vec::new();
        let mut ctor_tokens = Vec::new();
        for Field { attrs, ident, offset, width, traits } in &self.fields {
            let cfg_attrs = attrs.iter().filter(|attr| {
                attr.path.get_ident().and_then(|p| Some(p == "cfg")).unwrap_or(false)
            }).collect::<Vec<_>>();
            let field_snk = ident.to_string().to_snake_case();
            let mut field_psc = ident.to_string().to_pascal_case();
            if field_psc == "Val" {
                field_psc.push('_');
            }
            let field_psc = format_ident!("{}", field_psc);
            let field_ident = format_ident!("{}", unkeywordize(&field_snk));
            imports.extend(traits.iter().cloned());
            struct_tokens.push(quote! {
                #(#attrs)*
                pub #field_ident: #field_psc<#t>
            });
            ctor_tokens.push(quote! {
                #(#cfg_attrs)*
                #field_ident: ::drone_core::token::Token::take()
            });
            tokens.push(quote! {
                #(#attrs)*
                #[derive(Clone, Copy)]
                pub struct #field_psc<#t: ::drone_core::reg::tag::RegTag>(#t);

                #(#cfg_attrs)*
                unsafe impl<#t> ::drone_core::token::Token for #field_psc<#t>
                where
                    #t: ::drone_core::reg::tag::RegTag,
                {
                    #[inline]
                    unsafe fn take() -> Self {
                        #field_psc(#t::default())
                    }
                }

                #(#cfg_attrs)*
                impl<#t> ::drone_core::reg::field::RegField<#t> for #field_psc<#t>
                where
                    #t: ::drone_core::reg::tag::RegTag,
                {
                    type Reg = Reg<#t>;
                    type URegField = #field_psc<::drone_core::reg::tag::Urt>;
                    type SRegField = #field_psc<::drone_core::reg::tag::Srt>;
                    type CRegField = #field_psc<::drone_core::reg::tag::Crt>;

                    const OFFSET: usize = #offset;
                    const WIDTH: usize = #width;
                }
            });
            for ident in traits {
                tokens.push(quote! {
                    #(#cfg_attrs)*
                    impl<#t: ::drone_core::reg::tag::RegTag> #ident<#t> for #field_psc<#t> {}
                });
            }
            if width.base10_digits() == "1" {
                tokens.push(quote! {
                    #(#cfg_attrs)*
                    impl<#t> ::drone_core::reg::field::RegFieldBit<#t> for #field_psc<#t>
                    where
                        #t: ::drone_core::reg::tag::RegTag,
                    {
                    }
                });
                if traits.iter().any(|name| name == "RRRegField") {
                    tokens.push(quote! {
                        #[allow(clippy::len_without_is_empty)]
                        impl<'a, #t: ::drone_core::reg::tag::RegTag> Hold<'a, #t> {
                            #(#attrs)*
                            #[inline]
                            pub fn #field_ident(&self) -> bool {
                                ::drone_core::reg::field::RRRegFieldBit::read(
                                    &self.reg.#field_ident,
                                    &self.val,
                                )
                            }
                        }
                    });
                }
                if traits.iter().any(|name| name == "WWRegField") {
                    let set_field = format_ident!("set_{}", field_snk);
                    let clear_field = format_ident!("clear_{}", field_snk);
                    let toggle_field = format_ident!("toggle_{}", field_snk);
                    tokens.push(quote! {
                        #[allow(clippy::len_without_is_empty)]
                        impl<'a, #t: ::drone_core::reg::tag::RegTag> Hold<'a, #t> {
                            #(#attrs)*
                            #[inline]
                            pub fn #set_field(&mut self) -> &mut Self {
                                ::drone_core::reg::field::WWRegFieldBit::set(
                                    &self.reg.#field_ident,
                                    &mut self.val,
                                );
                                self
                            }

                            #(#attrs)*
                            #[inline]
                            pub fn #clear_field(&mut self) -> &mut Self {
                                ::drone_core::reg::field::WWRegFieldBit::clear(
                                    &self.reg.#field_ident,
                                    &mut self.val,
                                );
                                self
                            }

                            #(#attrs)*
                            #[inline]
                            pub fn #toggle_field(&mut self) -> &mut Self {
                                ::drone_core::reg::field::WWRegFieldBit::toggle(
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
                    impl<#t> ::drone_core::reg::field::RegFieldBits<#t> for #field_psc<#t>
                    where
                        #t: ::drone_core::reg::tag::RegTag,
                    {
                    }
                });
                if traits.iter().any(|name| name == "RRRegField") {
                    tokens.push(quote! {
                        #[allow(clippy::len_without_is_empty)]
                        impl<'a, #t: ::drone_core::reg::tag::RegTag> Hold<'a, #t> {
                            #(#attrs)*
                            #[inline]
                            pub fn #field_ident(&self) -> #val_ty {
                                ::drone_core::reg::field::RRRegFieldBits::read(
                                    &self.reg.#field_ident,
                                    &self.val,
                                )
                            }
                        }
                    });
                }
                if traits.iter().any(|name| name == "WWRegField") {
                    let write_field = format_ident!("write_{}", field_snk);
                    tokens.push(quote! {
                        #[allow(clippy::len_without_is_empty)]
                        impl<'a, #t: ::drone_core::reg::tag::RegTag> Hold<'a, #t> {
                            #(#attrs)*
                            #[inline]
                            pub fn #write_field(&mut self, bits: #val_ty) -> &mut Self {
                                ::drone_core::reg::field::WWRegFieldBits::write(
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
        if self.fields.is_empty() {
            struct_tokens.push(quote!(_marker: ::core::marker::PhantomData<#t>));
            ctor_tokens.push(quote!(_marker: ::core::marker::PhantomData));
        }
        for ident in &self.traits {
            tokens.push(quote! {
                impl<#t: ::drone_core::reg::tag::RegTag> #ident<#t> for Reg<#t> {}
            });
        }
        let imports = if imports.is_empty() {
            quote!()
        } else {
            let imports = imports.iter();
            quote!(use super::{#(#imports),*};)
        };
        let Variant { attrs, vis, address, reset, .. } = self;
        let reg_full = self.reg_full();

        quote! {
            #(#attrs)*
            #vis mod #reg_full {
                #imports
                use ::drone_core::bitfield::Bitfield;

                #(#attrs)*
                #[derive(Bitfield, Clone, Copy)]
                pub struct Val(#val_ty);

                #(#attrs)*
                #[derive(Clone, Copy)]
                pub struct Reg<#t: ::drone_core::reg::tag::RegTag> {
                    #(#struct_tokens),*
                }

                unsafe impl<#t: ::drone_core::reg::tag::RegTag> ::drone_core::token::Token for Reg<#t> {
                    #[inline]
                    unsafe fn take() -> Self {
                        Self { #(#ctor_tokens,)* }
                    }
                }

                impl<#t: ::drone_core::reg::tag::RegTag> ::drone_core::reg::Reg<#t> for Reg<#t> {
                    type Val = Val;
                    type UReg = Reg<::drone_core::reg::tag::Urt>;
                    type SReg = Reg<::drone_core::reg::tag::Srt>;
                    type CReg = Reg<::drone_core::reg::tag::Crt>;

                    const ADDRESS: usize = #address;
                    const RESET: #val_ty = #reset;

                    #[inline]
                    unsafe fn val_from(bits: #val_ty) -> Val {
                        Val(bits)
                    }
                }

                impl<'a, #t> ::drone_core::reg::RegRef<'a, #t> for Reg<#t>
                where
                    #t: ::drone_core::reg::tag::RegTag + 'a,
                {
                    type Hold = Hold<'a, #t>;

                    #[inline]
                    fn hold(&'a self, val: Self::Val) -> Self::Hold {
                        Hold { reg: self, val }
                    }
                }

                #(#attrs)*
                pub struct Hold<'a, #t: ::drone_core::reg::tag::RegTag> {
                    reg: &'a Reg<#t>,
                    val: Val,
                }

                impl<'a, #t> ::drone_core::reg::RegHold<'a, #t, Reg<#t>> for Hold<'a, #t>
                where
                    #t: ::drone_core::reg::tag::RegTag,
                {
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
        }
    }

    fn reg_full(&self) -> Ident {
        format_ident!(
            "{}_{}",
            self.block.to_string().to_snake_case(),
            self.ident.to_string().to_snake_case()
        )
    }
}

fn parse_traits(input: ParseStream<'_>) -> Result<Vec<Ident>> {
    let mut traits = Vec::new();
    let input2;
    braced!(input2 in input);
    while !input2.is_empty() {
        traits.push(input2.parse()?);
    }
    Ok(traits)
}

pub fn proc_macro(input: TokenStream) -> TokenStream {
    let Input { variants } = parse_macro_input!(input);
    let reg_tokens = variants.iter().map(Variant::generate).collect::<Vec<_>>();
    let mut variant_tokens = Vec::new();
    for (i, reg_src) in variants.iter().enumerate() {
        for (j, reg_dst) in variants.iter().enumerate() {
            if i == j {
                continue;
            }
            let t = format_ident!("_T");
            let mod_src = reg_src.reg_full();
            let mod_dst = reg_dst.reg_full();
            let into_variant = format_ident!(
                "into_{}_{}",
                reg_dst.block.to_string().to_snake_case(),
                reg_dst.ident.to_string().to_snake_case()
            );
            let doc = LitStr::new(
                &format!(
                    "Converts the token of variant `{}`, to a token of variant `{}`.",
                    mod_src, mod_dst
                ),
                Span::call_site(),
            );
            variant_tokens.push(quote! {
                impl<#t: ::drone_core::reg::tag::RegTag> #mod_src::Reg<#t> {
                    #[doc = #doc]
                    pub fn #into_variant(self) -> #mod_dst::Reg<#t> {
                        unsafe { ::drone_core::token::Token::take() }
                    }
                }
            });
        }
    }
    let expanded = quote! {
        #(#reg_tokens)*
        #(#variant_tokens)*
    };
    expanded.into()
}
