/// Returns a result of the I/O operation. Should be used inside [`AsyncFuture`]
/// context.
///
/// [`AsyncFuture`]: ../async/struct.AsyncFuture.html
#[macro_export]
macro_rules! ioawait {
  ($sess:ident . $($rest:tt)*) => {
    {
      #[allow(unused_imports)]
      use $crate::io::{Future, Responder};
      #[allow(unreachable_patterns, unreachable_code)]
      match await!($sess.$($rest)*) {
        Ok((sess, responder)) => {
          $sess = sess;
          Ok(responder.respond(&$sess))
        }
        Err((sess, error)) => {
          $sess = sess;
          Err(error)
        }
      }
    }
  }
}
