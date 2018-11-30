use syn::parse::{Parse, ParseStream, Result};
use syn::ExprPath;

/// Binding to extern static: `extern static Foo;`.
#[allow(missing_docs)]
pub struct ExternStatic {
  pub path: ExprPath,
}

impl Parse for ExternStatic {
  fn parse(input: ParseStream) -> Result<Self> {
    input.parse::<Token![extern]>()?;
    input.parse::<Token![static]>()?;
    let path = input.parse()?;
    input.parse::<Token![;]>()?;
    Ok(Self { path })
  }
}
