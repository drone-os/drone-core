use syn::parse::{Parse, ParseStream, Result};
use syn::ExprPath;

/// Binding to extern struct: `extern struct Foo;`.
#[allow(missing_docs)]
pub struct ExternStruct {
  pub path: ExprPath,
}

impl Parse for ExternStruct {
  fn parse(input: ParseStream) -> Result<Self> {
    input.parse::<Token![extern]>()?;
    input.parse::<Token![struct]>()?;
    let path = input.parse()?;
    input.parse::<Token![;]>()?;
    Ok(Self { path })
  }
}
