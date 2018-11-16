use fib::{self, Fiber, FiberState};
use futures::prelude::*;
use sync::spsc::ring::{channel, Receiver, SendError, SendErrorKind};
use thr::prelude::*;

/// A stream of results from another thread.
///
/// This stream can be created by the instance of [`Thread`](::thr::Thread).
#[must_use]
pub struct FiberStreamRing<I, E> {
  rx: Receiver<I, E>,
}

impl<I, E> FiberStreamRing<I, E> {
  /// Gracefully close this stream, preventing sending any future messages.
  #[inline]
  pub fn close(&mut self) {
    self.rx.close()
  }
}

impl<I, E> Stream for FiberStreamRing<I, E> {
  type Item = I;
  type Error = E;

  #[inline]
  fn poll_next(&mut self, cx: &mut task::Context) -> Poll<Option<I>, E> {
    self.rx.poll_next(cx)
  }
}

/// Adds a new ring stream fiber on the given `thr`.
pub fn add_stream_ring<T, U, O, F, I, E>(
  thr: T,
  capacity: usize,
  overflow: O,
  mut fib: F,
) -> FiberStreamRing<I, E>
where
  T: AsRef<U>,
  U: Thread,
  O: Fn(I) -> Result<(), E>,
  F: Fiber<Input = (), Yield = Option<I>, Return = Result<Option<I>, E>>,
  O: Send + 'static,
  F: Send + 'static,
  I: Send + 'static,
  E: Send + 'static,
{
  let (rx, mut tx) = channel(capacity);
  fib::add(thr, move || loop {
    if tx.is_canceled() {
      break;
    }
    match fib.resume(()) {
      FiberState::Yielded(None) => {}
      FiberState::Yielded(Some(value)) => match tx.send(value) {
        Ok(()) => {}
        Err(SendError { value, kind }) => match kind {
          SendErrorKind::Canceled => {
            break;
          }
          SendErrorKind::Overflow => match overflow(value) {
            Ok(()) => {}
            Err(err) => {
              tx.send_err(err).ok();
              break;
            }
          },
        },
      },
      FiberState::Complete(Ok(None)) => {
        break;
      }
      FiberState::Complete(Ok(Some(value))) => {
        tx.send(value).ok();
        break;
      }
      FiberState::Complete(Err(err)) => {
        tx.send_err(err).ok();
        break;
      }
    }
    yield;
  });
  FiberStreamRing { rx }
}

/// Adds a new ring stream fiber on the given `thr`. Overflows will be ignored.
pub fn add_stream_ring_skip<T, U, F, I, E>(
  thr: T,
  capacity: usize,
  fib: F,
) -> FiberStreamRing<I, E>
where
  T: AsRef<U>,
  U: Thread,
  F: Fiber<Input = (), Yield = Option<I>, Return = Result<Option<I>, E>>,
  F: Send + 'static,
  I: Send + 'static,
  E: Send + 'static,
{
  add_stream_ring(thr, capacity, |_| Ok(()), fib)
}

/// Adds a new ring stream fiber on the given `thr`. Overflows will overwrite.
pub fn add_stream_ring_overwrite<T, U, F, I, E>(
  thr: T,
  capacity: usize,
  mut fib: F,
) -> FiberStreamRing<I, E>
where
  T: AsRef<U>,
  U: Thread,
  F: Fiber<Input = (), Yield = Option<I>, Return = Result<Option<I>, E>>,
  F: Send + 'static,
  I: Send + 'static,
  E: Send + 'static,
{
  let (rx, mut tx) = channel(capacity);
  fib::add(thr, move || loop {
    if tx.is_canceled() {
      break;
    }
    match fib.resume(()) {
      FiberState::Yielded(None) => {}
      FiberState::Yielded(Some(value)) => match tx.send_overwrite(value) {
        Ok(()) => (),
        Err(_) => break,
      },
      FiberState::Complete(Ok(None)) => {
        break;
      }
      FiberState::Complete(Ok(Some(value))) => {
        tx.send_overwrite(value).ok();
        break;
      }
      FiberState::Complete(Err(err)) => {
        tx.send_err(err).ok();
        break;
      }
    }
    yield;
  });
  FiberStreamRing { rx }
}
