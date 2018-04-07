/// Returns a result of the I/O operation. Should be used inside
/// [`AsyncFuture`](AsyncFuture) context.
#[macro_export]
macro_rules! ioawait {
  ($sess:ident . $($tail:tt)*) => {{
    #[allow(unused_imports)]
    use $crate::io::{Future, Responder};
    #[allow(unreachable_code, unreachable_patterns, unused_assignments)]
    match await!($sess.$($tail)*) {
      Ok((sess, responder)) => {
        $sess = sess;
        Ok(Responder::respond(responder, &$sess))
      }
      Err((sess, err)) => {
        $sess = sess;
        Err(err)
      }
    }
  }};
}

/// Analogue of `try!` for [`io::Future`](io::Future) context.
#[macro_export]
macro_rules! iotry {
  ($sess:ident, $expr:expr) => {
    match $expr {
      Ok(val) => val,
      Err(err) => return Err(($sess, err.into())),
    }
  };
}

/// Applies `iotry!` to the result of `ioawait!`.
#[macro_export]
macro_rules! iotryawait {
  ($sess:ident . $($tail:tt)*) => {{
    iotry!($sess, ioawait!($sess.$($tail)*))
  }};
}
