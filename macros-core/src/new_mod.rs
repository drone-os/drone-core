use syn::synom::Synom;
use syn::{Attribute, Ident, Visibility};

/// Creates a new struct: `mod Foo;`.
#[allow(missing_docs)]
pub struct NewMod {
  pub attrs: Vec<Attribute>,
  pub vis: Visibility,
  pub ident: Ident,
}

impl Synom for NewMod {
  named!(parse -> Self, do_parse!(
    attrs: many0!(Attribute::parse_outer) >>
    vis: syn!(Visibility) >>
    keyword!(mod) >>
    ident: syn!(Ident) >>
    punct!(;) >>
    (NewMod { attrs, vis, ident })
  ));
}
