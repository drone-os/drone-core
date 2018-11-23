use fib::{Fiber, FiberState};
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

/// Unit stream extension to the thread token.
pub trait ThrStreamUnit<T: ThrAttach>: ThrToken<T> {
  /// Adds a new unit stream fiber.
  fn add_stream<O, F, E>(self, overflow: O, mut fib: F) -> FiberStreamUnit<E>
  where
    O: Fn() -> Result<(), E>,
    F: Fiber<Input = (), Yield = Option<()>, Return = Result<Option<()>, E>>,
    O: Send + 'static,
    F: Send + 'static,
    E: Send + 'static,
  {
    let (rx, mut tx) = channel();
    self.add(move || loop {
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

  /// Adds a new unit stream fiber. Overflows will be ignored.
  fn add_stream_skip<F, E>(self, fib: F) -> FiberStreamUnit<E>
  where
    F: Fiber<Input = (), Yield = Option<()>, Return = Result<Option<()>, E>>,
    F: Send + 'static,
    E: Send + 'static,
  {
    self.add_stream(|| Ok(()), fib)
  }
}

impl<T: ThrAttach, U: ThrToken<T>> ThrStreamUnit<T> for U {}
