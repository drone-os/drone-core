use drone_macros_core::{unkeywordize, CfgCond, CfgCondExt};
use inflector::Inflector;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    braced,
    parse::{Parse, ParseStream, Result},
    parse_macro_input, Attribute, Ident, Path, Token,
};

const MACRO_PREFIX: &str = "periph_";
const STRUCT_SUFFIX: &str = "Periph";

struct PeriphSingular {
    macro_attrs: Vec<Attribute>,
    macro_ident: Ident,
    struct_attrs: Vec<Attribute>,
    struct_ident: Ident,
    root_path: Path,
    macro_root_path: Option<Path>,
    blocks: Vec<Block>,
}

struct Block {
    ident: Ident,
    regs: Vec<Reg>,
}

struct Reg {
    features: CfgCond,
    ident: Ident,
    fields: Vec<Field>,
}

struct Field {
    features: CfgCond,
    ident: Ident,
}

impl Parse for PeriphSingular {
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
        let struct_ident = input.parse::<Ident>()?;
        if !struct_ident.to_string().ends_with(STRUCT_SUFFIX) {
            return Err(input.error(format!(
                "Expected an ident which ends with `{}`",
                STRUCT_SUFFIX
            )));
        }
        input.parse::<Token![;]>()?;
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
        let mut fields = Vec::new();
        if input.peek(Token![;]) {
            input.parse::<Token![;]>()?;
        } else {
            let content;
            braced!(content in input);
            while !content.is_empty() {
                fields.push(content.parse()?);
            }
        }
        Ok(Self {
            features,
            ident,
            fields,
        })
    }
}

impl Parse for Field {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let features = input.parse()?;
        let ident = input.parse()?;
        input.parse::<Token![;]>()?;
        Ok(Self { features, ident })
    }
}

pub fn proc_macro(input: TokenStream) -> TokenStream {
    let PeriphSingular {
        macro_attrs,
        macro_ident,
        struct_attrs,
        struct_ident,
        root_path,
        macro_root_path,
        blocks,
    } = &parse_macro_input!(input as PeriphSingular);
    let mut tokens = Vec::new();
    let mut periph_tokens = Vec::new();
    let mut macro_tokens = Vec::new();
    for Block {
        ident: block_ident,
        regs,
    } in blocks
    {
        let block_snk = block_ident.to_string().to_snake_case();
        let block_ident = format_ident!("{}", unkeywordize(&block_snk));
        for Reg {
            features: reg_features,
            ident: reg_ident,
            fields,
        } in regs
        {
            let reg_snk = reg_ident.to_string().to_snake_case();
            let reg_ident = format_ident!("{}", unkeywordize(&reg_snk));
            let block_reg_snk = format_ident!("{}_{}", block_snk, reg_snk);
            let reg_attrs = reg_features.attrs();
            if fields.is_empty() {
                periph_tokens.push(quote! {
                    #[allow(missing_docs)]
                    #reg_attrs
                    pub #block_reg_snk: #root_path::#block_ident::#reg_ident::Reg<
                        ::drone_core::reg::tag::Srt,
                    >
                });
                macro_tokens.push((
                    reg_features.clone(),
                    quote!(#block_reg_snk: $reg.#block_reg_snk),
                ));
            } else {
                for Field {
                    features: field_features,
                    ident: field_ident,
                } in fields
                {
                    let field_snk = field_ident.to_string().to_snake_case();
                    let field_psc = format_ident!("{}", field_ident.to_string().to_pascal_case());
                    let field_ident = format_ident!("{}", unkeywordize(&field_snk));
                    let block_reg_field_snk =
                        format_ident!("{}_{}_{}", block_snk, reg_snk, field_snk);
                    let mut features = CfgCond::default();
                    features.add_clause(&reg_features);
                    features.add_clause(&field_features);
                    let field_attrs = features.attrs();
                    periph_tokens.push(quote! {
                        #[allow(missing_docs)]
                        #field_attrs
                        pub #block_reg_field_snk: #root_path::#block_ident::#reg_ident::#field_psc<
                            ::drone_core::reg::tag::Srt,
                        >
                    });
                    macro_tokens.push((
                        features,
                        quote!(#block_reg_field_snk: $reg.#block_reg_snk.#field_ident),
                    ));
                }
            }
        }
    }
    for (features, macro_tokens) in macro_tokens.as_slice().transpose() {
        let attrs = features.attrs();
        let macro_root_path = macro_root_path.iter();
        tokens.push(quote! {
            #attrs
            #(#macro_attrs)*
            #[macro_export]
            macro_rules! #macro_ident {
                ($reg:ident) => {
                    $crate#(#macro_root_path)*::#struct_ident {
                        #(#macro_tokens,)*
                    }
                };
            }
        });
    }
    let expanded = quote! {
        #(#struct_attrs)*
        pub struct #struct_ident {
            #(#periph_tokens,)*
        }

        #(#tokens)*
    };
    expanded.into()
}
