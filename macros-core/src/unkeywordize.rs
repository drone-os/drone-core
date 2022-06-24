use once_cell::sync::Lazy;
use regex::Regex;

static KEYWORDS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?x)
            ^ ( as | break | const | continue | crate | else | enum | extern | false | fn | for | if
            | impl | in | let | loop | match | mod | move | mut | pub | ref | return | Self | self |
            static | struct | super | trait | true | type | unsafe | use | where | while | abstract
            | alignof | become | box | do | final | macro | offsetof | override | priv | proc | pure
            | sizeof | typeof | unsized | virtual | yield ) $
        ",
    )
    .unwrap()
});

/// Inserts an underscore at the end of the string if the string is a reserved
/// keyword.
pub fn unkeywordize<T: AsRef<str>>(ident: T) -> String {
    let mut ident = ident.as_ref().to_string();
    if KEYWORDS.is_match(ident.as_ref()) {
        ident.push('_');
    }
    ident
}
