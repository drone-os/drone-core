use regex::Regex;

lazy_static! {
  static ref RESERVED: Regex = Regex::new(r"(?x)
    ^ ( as | break | const | continue | crate | else | enum | extern | false |
    fn | for | if | impl | in | let | loop | match | mod | move | mut | pub |
    ref | return | Self | self | static | struct | super | trait | true | type |
    unsafe | use | where | while | abstract | alignof | become | box | do |
    final | macro | offsetof | override | priv | proc | pure | sizeof | typeof |
    unsized | virtual | yield ) $
  ").unwrap();
}

/// Inserts underscore symbol at the beginning of the string if string is a
/// reserved keyword.
pub fn reserved_check(mut name: String) -> String {
  if RESERVED.is_match(&name) {
    name.insert(0, '_');
  }
  name
}
