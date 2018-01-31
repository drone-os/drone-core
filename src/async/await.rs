/// Returns a result of the future. Should be used inside [`AsyncFuture`]
/// context.
///
/// [`AsyncFuture`]: struct.AsyncFuture.html
#[macro_export]
macro_rules! await {
  ($future:expr) => {
    {
      let mut future = $future;
      loop {
        let result = future.poll();
        #[allow(unreachable_patterns, unreachable_code)]
        match result {
          Ok(Async::NotReady) => {
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
    }
  }
}

/// Asynchronously iterates over a stream. Should be used inside [`AsyncFuture`]
/// context.
///
/// [`AsyncFuture`]: struct.AsyncFuture.html
#[macro_export]
macro_rules! await_for {
  ($pat:pat in $expr:expr => $block:block) => {
    {
      let mut stream = $expr;
      loop {
        let $pat = {
          let result = stream.poll()?;
          match result {
            Async::NotReady => {
              yield;
              continue;
            }
            Async::Ready(Some(value)) => {
              value
            }
            Async::Ready(None) => {
              break;
            }
          }
        };
        $block
      }
    }
  }
}
