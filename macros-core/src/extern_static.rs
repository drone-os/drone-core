use syn::parse::{Parse, ParseStream, Result};
use syn::Ident;

/// Binding to extern static: `extern static Foo;`.
#[allow(missing_docs)]
pub struct ExternStatic {
  pub ident: Ident,
}

impl Parse for ExternStatic {
  fn parse(input: ParseStream) -> Result<Self> {
    input.parse::<Token![extern]>()?;
    input.parse::<Token![static]>()?;
    let ident = input.parse()?;
    input.parse::<Token![;]>()?;
    Ok(Self { ident })
  }
}
