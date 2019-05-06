/// A macro to await a future on an async call.
#[macro_export]
macro_rules! awt {
  ($expr:expr) => {{
    let mut pinned = $expr;
    loop {
      match $crate::asnc::poll_with_context(unsafe {
        $crate::asnc::__rt::Pin::new_unchecked(&mut pinned)
      }) {
        $crate::asnc::__rt::Poll::Ready(x) => break x,
        $crate::asnc::__rt::Poll::Pending => yield,
      }
    }
  }};
}

/// A macro to await a stream item on an async call.
#[macro_export]
macro_rules! awt_next {
  ($expr:expr) => {
    loop {
      match $crate::asnc::poll_next_with_context($crate::asnc::__rt::Pin::new(
        &mut $expr,
      )) {
        $crate::asnc::__rt::Poll::Ready(x) => break x,
        $crate::asnc::__rt::Poll::Pending => yield,
      }
    }
  };
}

/// A macro to await stream items on an async call.
#[macro_export]
macro_rules! awt_for {
  ($pat:pat in $expr:expr => $block:block) => {{
    let mut pinned = $expr;
    loop {
      let $pat = {
        match $crate::asnc::poll_next_with_context(unsafe {
          $crate::asnc::__rt::Pin::new_unchecked(&mut pinned)
        }) {
          $crate::asnc::__rt::Poll::Ready(
            $crate::asnc::__rt::Option::Some(x),
          ) => x,
          $crate::asnc::__rt::Poll::Ready($crate::asnc::__rt::Option::None) => {
            break;
          }
          $crate::asnc::__rt::Poll::Pending => {
            yield;
            continue;
          }
        }
      };
      $block
    }
  }};
}
