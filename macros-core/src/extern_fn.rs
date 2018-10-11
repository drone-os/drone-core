use syn::parse::{Parse, ParseStream, Result};
use syn::Ident;

/// Binding to extern function: `extern fn Foo;`.
#[allow(missing_docs)]
pub struct ExternFn {
  pub ident: Ident,
}

impl Parse for ExternFn {
  fn parse(input: ParseStream) -> Result<Self> {
    input.parse::<Token![extern]>()?;
    input.parse::<Token![fn]>()?;
    let ident = input.parse()?;
    input.parse::<Token![;]>()?;
    Ok(Self { ident })
  }
}
