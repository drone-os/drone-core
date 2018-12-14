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

#[macro_export]
macro_rules! new_def_ident {
  ($fmt:expr, $($args:tt)*) => {
    ::syn::Ident::new(
      &format!($fmt, $($args)*),
      ::proc_macro2::Span::def_site(),
    )
  };
  ($fmt:expr) => {
    new_def_ident!($fmt,)
  };
}

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
