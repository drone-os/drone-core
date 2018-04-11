use syn::synom::Synom;
use syn::Ident;

/// Binding to extern static: `extern static Foo;`.
#[allow(missing_docs)]
pub struct ExternStatic {
  pub ident: Ident,
}

impl Synom for ExternStatic {
  named!(parse -> Self, do_parse!(
    keyword!(extern) >>
    keyword!(static) >>
    ident: syn!(Ident) >>
    punct!(;) >>
    (ExternStatic { ident })
  ));
}
