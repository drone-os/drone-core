use regex::Regex;
use std::borrow::Cow;

lazy_static! {
  static ref KEYWORDS: Regex = Regex::new(
    r"(?x)
      ^ ( as | break | const | continue | crate | else | enum | extern | false |
      fn | for | if | impl | in | let | loop | match | mod | move | mut | pub |
      ref | return | Self | self | static | struct | super | trait | true | type
      | unsafe | use | where | while | abstract | alignof | become | box | do |
      final | macro | offsetof | override | priv | proc | pure | sizeof | typeof
      | unsized | virtual | yield ) $
    "
  ).unwrap();
}

/// Inserts an underscore at the beginning of the string if the string is a
/// reserved keyword.
pub fn unkeywordize(mut ident: Cow<str>) -> Cow<str> {
  if KEYWORDS.is_match(&ident) {
    ident.to_mut().insert(0, '_');
  }
  ident
}
