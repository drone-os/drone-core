use syn::synom::Synom;
use syn::{Attribute, Ident, Visibility};

/// Creates a new static: `static Foo;`.
#[allow(missing_docs)]
pub struct NewStatic {
  pub attrs: Vec<Attribute>,
  pub vis: Visibility,
  pub ident: Ident,
}

impl Synom for NewStatic {
  named!(parse -> Self, do_parse!(
    attrs: many0!(Attribute::parse_outer) >>
    vis: syn!(Visibility) >>
    keyword!(static) >>
    ident: syn!(Ident) >>
    punct!(;) >>
    (NewStatic { attrs, vis, ident })
  ));
}
