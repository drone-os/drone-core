/// Creates call site [`Ident`](::syn::Ident) using interpolation of runtime
/// expressions.
#[macro_export]
macro_rules! new_ident {
    ($fmt:expr, $($args:tt)*) => {
        ::syn::Ident::new(
            &format!($fmt, $($args)*),
            ::proc_macro2::Span::call_site(),
        )
    };
    ($fmt:expr) => {
        new_ident!($fmt,)
    };
}

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
