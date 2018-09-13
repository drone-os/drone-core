use syn::parse::{Parse, ParseStream, Result};
use syn::Ident;

/// Binding to extern struct: `extern struct Foo;`.
#[allow(missing_docs)]
pub struct ExternStruct {
  pub ident: Ident,
}

impl Parse for ExternStruct {
  fn parse(input: ParseStream) -> Result<Self> {
    input.parse::<Token![extern]>()?;
    input.parse::<Token![struct]>()?;
    let ident = input.parse()?;
    input.parse::<Token![;]>()?;
    Ok(Self { ident })
  }
}
