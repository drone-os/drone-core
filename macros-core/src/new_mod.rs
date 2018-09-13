use syn::parse::{Parse, ParseStream, Result};
use syn::{Attribute, Ident, Visibility};

/// Creates a new struct: `mod Foo;`.
#[allow(missing_docs)]
pub struct NewMod {
  pub attrs: Vec<Attribute>,
  pub vis: Visibility,
  pub ident: Ident,
}

impl Parse for NewMod {
  fn parse(input: ParseStream) -> Result<Self> {
    let attrs = input.call(Attribute::parse_outer)?;
    let vis = input.parse()?;
    input.parse::<Token![mod]>()?;
    let ident = input.parse()?;
    input.parse::<Token![;]>()?;
    Ok(Self { attrs, vis, ident })
  }
}
