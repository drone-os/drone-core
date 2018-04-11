/// Returns a result of the future. Should be used inside
/// [`AsyncFuture`](AsyncFuture) context.
#[macro_export]
macro_rules! await {
  ($future:expr) => {{
    let mut future = $future;
    loop {
      let poll = $crate::thr::__current_task().__in_cx(|cx| future.poll(cx));
      #[allow(unreachable_patterns, unreachable_code)]
      match poll {
        Ok(Async::Pending) => {
          yield;
        }
        Ok(Async::Ready(ready)) => {
          break Ok(ready);
        }
        Err(err) => {
          break Err(err);
        }
      }
    }
  }};
}

/// Asynchronously iterates over a stream. Should be used inside
/// [`AsyncFuture`](AsyncFuture) context.
#[macro_export]
macro_rules! await_for {
  ($pat:pat in $expr:expr => $block:block) => {{
    let mut stream = $expr;
    loop {
      let $pat = {
        let poll =
          $crate::thr::__current_task().__in_cx(|cx| stream.poll_next(cx));
        match poll? {
          Async::Pending => {
            yield;
            continue;
          }
          Async::Ready(Some(value)) => value,
          Async::Ready(None) => {
            break;
          }
        }
      };
      $block
    }
  }};
}
