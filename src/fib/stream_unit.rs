use fib::{self, Fiber, FiberState};
use futures::prelude::*;
use sync::spsc::unit::{channel, Receiver, SendError};
use thr::prelude::*;

/// A stream of results from another thread.
///
/// This stream can be created by the instance of [`Thread`](::thr::Thread).
#[must_use]
pub struct FiberStreamUnit<E> {
  rx: Receiver<E>,
}

impl<E> FiberStreamUnit<E> {
  /// Gracefully close this stream, preventing sending any future messages.
  #[inline]
  pub fn close(&mut self) {
    self.rx.close()
  }
}

impl<E> Stream for FiberStreamUnit<E> {
  type Item = ();
  type Error = E;

  #[inline]
  fn poll_next(&mut self, cx: &mut task::Context) -> Poll<Option<()>, E> {
    self.rx.poll_next(cx)
  }
}

/// Adds a new unit stream fiber on the given `thr`.
pub fn add_stream<T, U, O, F, E>(
  thr: T,
  overflow: O,
  mut fib: F,
) -> FiberStreamUnit<E>
where
  T: AsRef<U>,
  U: Thread,
  O: Fn() -> Result<(), E>,
  F: Fiber<Input = (), Yield = Option<()>, Return = Result<Option<()>, E>>,
  O: Send + 'static,
  F: Send + 'static,
  E: Send + 'static,
{
  let (rx, mut tx) = channel();
  fib::add(thr, move || loop {
    if tx.is_canceled() {
      break;
    }
    match fib.resume(()) {
      FiberState::Yielded(None) => {}
      FiberState::Yielded(Some(())) => match tx.send() {
        Ok(()) => {}
        Err(SendError::Canceled) => {
          break;
        }
        Err(SendError::Overflow) => match overflow() {
          Ok(()) => {}
          Err(err) => {
            tx.send_err(err).ok();
            break;
          }
        },
      },
      FiberState::Complete(Ok(None)) => {
        break;
      }
      FiberState::Complete(Ok(Some(()))) => {
        tx.send().ok();
        break;
      }
      FiberState::Complete(Err(err)) => {
        tx.send_err(err).ok();
        break;
      }
    }
    yield;
  });
  FiberStreamUnit { rx }
}

/// Adds a new unit stream fiber on the given `thr`. Overflows will be ignored.
pub fn add_stream_skip<T, U, F, E>(thr: T, fib: F) -> FiberStreamUnit<E>
where
  T: AsRef<U>,
  U: Thread,
  F: Fiber<Input = (), Yield = Option<()>, Return = Result<Option<()>, E>>,
  F: Send + 'static,
  E: Send + 'static,
{
  add_stream(thr, || Ok(()), fib)
}
