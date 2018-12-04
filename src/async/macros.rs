/// Returns a result of the future. Should be used inside
/// [`async`](fn@async) context.
#[macro_export]
macro_rules! await {
  ($future:expr) => {{
    let mut future = $future;
    loop {
      let poll = $crate::thr::__current_task().__in_cx(|cx| {
        #[allow(unused_imports)]
        use $crate::async::__rt::Future;
        future.poll(cx)
      });
      #[allow(unreachable_patterns, unreachable_code)]
      match poll {
        $crate::async::__rt::Result::Ok(
          $crate::async::__rt::Async::Pending,
        ) => {
          yield;
        }
        $crate::async::__rt::Result::Ok($crate::async::__rt::Async::Ready(
          ready,
        )) => {
          break $crate::async::__rt::Result::Ok(ready);
        }
        $crate::async::__rt::Result::Err(err) => {
          break $crate::async::__rt::Result::Err(err);
        }
      }
    }
  }};
}

/// Returns next item from the stream.  Should be used inside
/// [`async`](fn@async) context.
#[macro_export]
macro_rules! await_item {
  ($stream:expr) => {
    loop {
      let poll = $crate::thr::__current_task().__in_cx(|cx| {
        #[allow(unused_imports)]
        use $crate::async::__rt::Stream;
        $stream.poll_next(cx)
      });
      #[allow(unreachable_patterns, unreachable_code)]
      match poll {
        $crate::async::__rt::Result::Ok(
          $crate::async::__rt::Async::Pending,
        ) => {
          yield;
        }
        $crate::async::__rt::Result::Ok($crate::async::__rt::Async::Ready(
          ready,
        )) => {
          break $crate::async::__rt::Result::Ok(ready);
        }
        $crate::async::__rt::Result::Err(err) => {
          break $crate::async::__rt::Result::Err(err);
        }
      }
    }
  };
}

/// Asynchronously iterates over a stream. Should be used inside
/// [`async`](fn@async) context.
#[macro_export]
macro_rules! await_for {
  ($pat:pat in $expr:expr => $block:block) => {{
    let mut stream = $expr;
    loop {
      let $pat = {
        let poll = $crate::thr::__current_task().__in_cx(|cx| {
          #[allow(unused_imports)]
          use $crate::async::__rt::Stream;
          stream.poll_next(cx)
        });
        match poll? {
          $crate::async::__rt::Async::Pending => {
            yield;
            continue;
          }
          $crate::async::__rt::Async::Ready(
            $crate::async::__rt::Option::Some(value),
          ) => value,
          $crate::async::__rt::Async::Ready(
            $crate::async::__rt::Option::None,
          ) => {
            break;
          }
        }
      };
      $block
    }
  }};
}
