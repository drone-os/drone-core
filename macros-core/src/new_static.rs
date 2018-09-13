use syn::parse::{Parse, ParseStream, Result};
use syn::{Attribute, Ident, Visibility};

/// Creates a new static: `static Foo;`.
#[allow(missing_docs)]
pub struct NewStatic {
  pub attrs: Vec<Attribute>,
  pub vis: Visibility,
  pub ident: Ident,
}

impl Parse for NewStatic {
  fn parse(input: ParseStream) -> Result<Self> {
    let attrs = input.call(Attribute::parse_outer)?;
    let vis = input.parse()?;
    input.parse::<Token![static]>()?;
    let ident = input.parse()?;
    input.parse::<Token![;]>()?;
    Ok(Self { attrs, vis, ident })
  }
}
