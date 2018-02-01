/// Returns a result of the I/O operation. Should be used inside [`AsyncFuture`]
/// context.
///
/// [`AsyncFuture`]: ../async/struct.AsyncFuture.html
#[macro_export]
macro_rules! ioawait {
  ($sess:ident . $($rest:tt)*) => {
    {
      #[allow(unreachable_patterns, unreachable_code)]
      match await!($sess.$($rest)*) {
        Ok((sess, result)) => {
          $sess = sess;
          Ok(result(&$sess))
        }
        Err((sess, error)) => {
          $sess = sess;
          Err(error)
        }
      }
    }
  }
}
