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
