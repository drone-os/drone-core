use syn::Ident;
use syn::synom::Synom;

/// Binding to extern struct: `extern struct Foo;`.
#[allow(missing_docs)]
pub struct ExternStruct {
  pub ident: Ident,
}

impl Synom for ExternStruct {
  named!(parse -> Self, do_parse!(
    keyword!(extern) >>
    keyword!(struct) >>
    ident: syn!(Ident) >>
    punct!(;) >>
    (ExternStruct { ident })
  ));
}
