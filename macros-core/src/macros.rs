/// Unconditionally causes parsing to fail with the given error message.
#[macro_export]
macro_rules! compile_error {
    ($fmt:expr, $($args:tt)*) => {
        return ::syn::parse::Error::new(
            ::proc_macro2::Span::call_site(),
            format!($fmt, $($args)*),
        )
        .to_compile_error()
        .into()
    };
    ($fmt:expr) => {
        compile_error!($fmt,)
    };
}

/// Parses an identifier with a specific value, or throws an error otherwise.
#[macro_export]
macro_rules! parse_ident {
    ($input:ident, $value:expr) => {
        if $input.parse::<::syn::Ident>()? != $value {
            return Err($input.error(format!("Expected `{}`", $value)));
        }
    };
}
