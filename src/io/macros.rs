/// Returns a result of the I/O operation. Should be used inside [`AsyncFuture`]
/// context.
///
/// [`AsyncFuture`]: ../async/struct.AsyncFuture.html
#[macro_export]
macro_rules! ioawait {
  ($sess:ident . $($tail:tt)*) => {
    {
      #[allow(unused_imports)]
      use $crate::io::{Future, Responder};
      #[allow(unreachable_patterns, unreachable_code)]
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
    }
  }
}

/// Analogue of `try!` for [`io::Future`] context.
///
/// [`io::Future`]: trait.Future.html
#[macro_export]
macro_rules! iotry {
  (ioawait!($sess:ident . $($tail:tt)*)) => {
    iotry!($sess, ioawait!($sess.$($tail)*))
  };

  ($sess:ident, $expr:expr) => {
    match $expr {
      Ok(val) => val,
      Err(err) => {
        return Err(($sess, err.into()))
      }
    }
  };
}
