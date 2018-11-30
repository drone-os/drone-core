use syn::parse::{Parse, ParseStream, Result};
use syn::ExprPath;

/// Binding to extern function: `extern fn Foo;`.
#[allow(missing_docs)]
pub struct ExternFn {
  pub path: ExprPath,
}

impl Parse for ExternFn {
  fn parse(input: ParseStream) -> Result<Self> {
    input.parse::<Token![extern]>()?;
    input.parse::<Token![fn]>()?;
    let path = input.parse()?;
    input.parse::<Token![;]>()?;
    Ok(Self { path })
  }
}
