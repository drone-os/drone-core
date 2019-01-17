use drone_macros_core::{new_ident, unkeywordize, CfgFeatures, CfgFeaturesExt};
use inflector::Inflector;
use proc_macro::TokenStream;
use quote::quote;
use syn::{
  braced,
  parse::{Parse, ParseStream, Result},
  parse_macro_input, Attribute, Ident, Path, Token, Visibility,
};

struct ResOne {
  attrs: Vec<Attribute>,
  vis: Visibility,
  ident: Ident,
  root_path: Path,
  macro_root_path: Option<Path>,
  blocks: Vec<Block>,
}

struct Block {
  ident: Ident,
  regs: Vec<Reg>,
}

struct Reg {
  features: CfgFeatures,
  ident: Ident,
  fields: Vec<Field>,
}

struct Field {
  features: CfgFeatures,
  ident: Ident,
}

impl Parse for ResOne {
  fn parse(input: ParseStream) -> Result<Self> {
    let attrs = input.call(Attribute::parse_outer)?;
    let vis = input.parse()?;
    input.parse::<Token![struct]>()?;
    let ident = input.parse()?;
    input.parse::<Token![;]>()?;
    let root_path = input.parse()?;
    input.parse::<Token![;]>()?;
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
      attrs,
      vis,
      ident,
      root_path,
      macro_root_path,
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
  fn parse(input: ParseStream) -> Result<Self> {
    let features = input.parse()?;
    let ident = input.parse()?;
    input.parse::<Token![;]>()?;
    Ok(Self { features, ident })
  }
}

pub fn proc_macro(input: TokenStream) -> TokenStream {
  let ResOne {
    attrs,
    vis,
    ident,
    root_path,
    macro_root_path,
    blocks,
  } = &parse_macro_input!(input as ResOne);
  let mut tokens = Vec::new();
  let mut res_tokens = Vec::new();
  let mut macro_tokens = Vec::new();
  for Block {
    ident: block_ident,
    regs,
  } in blocks
  {
    let block_snk = block_ident.to_string().to_snake_case();
    let block_ident =
      new_ident!("{}", unkeywordize(block_snk.to_string().into()));
    for Reg {
      features: reg_features,
      ident: reg_ident,
      fields,
    } in regs
    {
      let reg_snk = reg_ident.to_string().to_snake_case();
      let reg_ident =
        new_ident!("{}", unkeywordize(reg_snk.to_string().into()));
      let block_reg_snk = new_ident!("{}_{}", block_snk, reg_snk);
      let reg_attrs = &reg_features.attrs();
      if fields.is_empty() {
        res_tokens.push(quote! {
          #(#reg_attrs)*
          pub #block_reg_snk: #root_path::#block_ident::#reg_ident::Reg<
            ::drone_core::reg::Srt,
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
          let field_psc =
            new_ident!("{}", field_ident.to_string().to_pascal_case());
          let field_ident =
            new_ident!("{}", unkeywordize(field_snk.clone().into()));
          let block_reg_field_snk =
            new_ident!("{}_{}_{}", block_snk, reg_snk, field_snk);
          let mut features = CfgFeatures::default();
          features.add_clause(&reg_features);
          features.add_clause(&field_features);
          let field_attrs = &features.attrs();
          res_tokens.push(quote! {
            #(#field_attrs)*
            pub #block_reg_field_snk:
              #root_path::#block_ident::#reg_ident::#field_psc<
                ::drone_core::reg::Srt,
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
  let res_macro = new_ident!("res_{}", ident.to_string().to_snake_case());
  let res_struct = new_ident!("{}Res", ident);
  for (features, macro_tokens) in macro_tokens.as_slice().transpose() {
    let attrs = &features.attrs();
    let doc = format!(
      "Acquire an instance of [`{}`] from the given `$reg` tokens",
      res_struct
    );
    tokens.push(quote! {
      #(#attrs)*
      #[doc = #doc]
      #[macro_export]
      macro_rules! #res_macro {
        ($reg:ident) => {
          $crate#(::#macro_root_path)*::#res_struct {
            #(#macro_tokens,)*
          }
        };
      }
    });
  }
  let expanded = quote! {
    #(#attrs)*
    #[allow(missing_docs)]
    #vis struct #res_struct {
      #(#res_tokens,)*
    }

    #(#tokens)*
  };
  expanded.into()
}
