/// Returns a result of the I/O operation. Should be used inside [`AsyncFuture`]
/// context.
///
/// [`AsyncFuture`]: ../async/struct.AsyncFuture.html
#[macro_export]
macro_rules! io_await {
  ($operation:ty, $sess:ident $(, $arg:expr)*) => {
    {
      #[allow(unused_imports)]
      use $crate::io::Operation;
      type Operator = $operation;
      #[cfg_attr(feature = "clippy", allow(double_parens))]
      #[allow(unused_parens)]
      let operator = Operator::new(($($arg),*));
      #[allow(unreachable_patterns, unreachable_code)]
      match await!(operator.operate($sess)) {
        Ok(sess) => {
          $sess = sess;
          Ok(operator.respond(&$sess))
        }
        Err($crate::io::Error { sess, kind }) => {
          $sess = sess;
          Err(kind)
        }
      }
    }
  }
}
