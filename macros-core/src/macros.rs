/// Matches the result of `syn::parse`. In case of `Ok` variant, the expression
/// has the value of the wrapped value. In case of `Err` variant, it retrieves
/// the inner error, emits its message on the given span, and immediately
/// returns an empty `TokenStream`.
#[macro_export]
macro_rules! try_parse {
  ($span:expr, $input:expr) => {{
    let span = $span;
    match ::syn::parse($input) {
      Ok(value) => value,
      Err(err) => return $crate::emit_parse_err(span, err),
    }
  }};
}

/// Matches the result of `syn::parse2`. In case of `Ok` variant, the expression
/// has the value of the wrapped value. In case of `Err` variant, it retrieves
/// the inner error, emits its message on the given span, and immediately
/// returns an empty `TokenStream`.
#[macro_export]
macro_rules! try_parse2 {
  ($span:expr, $input:expr) => {{
    let span = $span;
    match ::syn::parse2($input) {
      Ok(value) => value,
      Err(err) => return $crate::emit_parse_err(span, err),
    }
  }};
}
