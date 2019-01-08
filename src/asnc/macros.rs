/// Returns a result of the future. Should be used inside [`asnc`](fn@asnc)
/// context.
#[macro_export]
macro_rules! awt {
  ($future:expr) => {{
    let mut future = $future;
    loop {
      let poll = $crate::thr::__current_task().__in_cx(|cx| {
        #[allow(unused_imports)]
        use $crate::asnc::__rt::Future;
        future.poll(cx)
      });
      #[allow(unreachable_patterns, unreachable_code)]
      match poll {
        $crate::asnc::__rt::Result::Ok($crate::asnc::__rt::Async::Pending) => {
          yield;
        }
        $crate::asnc::__rt::Result::Ok($crate::asnc::__rt::Async::Ready(
          ready,
        )) => {
          break $crate::asnc::__rt::Result::Ok(ready);
        }
        $crate::asnc::__rt::Result::Err(err) => {
          break $crate::asnc::__rt::Result::Err(err);
        }
      }
    }
  }};
}

/// Returns next item from the stream. Should be used inside [`asnc`](fn@asnc)
/// context.
#[macro_export]
macro_rules! awt_item {
  ($stream:expr) => {
    loop {
      let poll = $crate::thr::__current_task().__in_cx(|cx| {
        #[allow(unused_imports)]
        use $crate::asnc::__rt::Stream;
        $stream.poll_next(cx)
      });
      #[allow(unreachable_patterns, unreachable_code)]
      match poll {
        $crate::asnc::__rt::Result::Ok($crate::asnc::__rt::Async::Pending) => {
          yield;
        }
        $crate::asnc::__rt::Result::Ok($crate::asnc::__rt::Async::Ready(
          ready,
        )) => {
          break $crate::asnc::__rt::Result::Ok(ready);
        }
        $crate::asnc::__rt::Result::Err(err) => {
          break $crate::asnc::__rt::Result::Err(err);
        }
      }
    }
  };
}

/// Asynchronously iterates over a stream. Should be used inside
/// [`asnc`](fn@asnc) context.
#[macro_export]
macro_rules! awt_for {
  ($pat:pat in $expr:expr => $block:block) => {{
    let mut stream = $expr;
    loop {
      let $pat = {
        let poll = $crate::thr::__current_task().__in_cx(|cx| {
          #[allow(unused_imports)]
          use $crate::asnc::__rt::Stream;
          stream.poll_next(cx)
        });
        match poll? {
          $crate::asnc::__rt::Async::Pending => {
            yield;
            continue;
          }
          $crate::asnc::__rt::Async::Ready(
            $crate::asnc::__rt::Option::Some(value),
          ) => value,
          $crate::asnc::__rt::Async::Ready(
            $crate::asnc::__rt::Option::None,
          ) => {
            break;
          }
        }
      };
      $block
    }
  }};
}
