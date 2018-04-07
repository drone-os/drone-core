use syn::synom::Synom;
use syn::{Attribute, Ident, Visibility};

/// Creates a new struct: `struct Foo;`.
#[allow(missing_docs)]
pub struct NewStruct {
  pub attrs: Vec<Attribute>,
  pub vis: Visibility,
  pub ident: Ident,
}

impl Synom for NewStruct {
  named!(parse -> Self, do_parse!(
    attrs: many0!(Attribute::parse_outer) >>
    vis: syn!(Visibility) >>
    keyword!(struct) >>
    ident: syn!(Ident) >>
    punct!(;) >>
    (NewStruct { attrs, vis, ident })
  ));
}
