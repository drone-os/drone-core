use crate::{
  fib::{Fiber, FiberState},
  sync::spsc::ring::{channel, Receiver, SendError, SendErrorKind},
  thr::prelude::*,
};
use futures::prelude::*;

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

/// Ring stream extension to the thread token.
pub trait ThrStreamRing<T: ThrAttach>: ThrToken<T> {
  /// Adds a new ring stream fiber.
  fn add_stream_ring<O, F, I, E>(
    self,
    capacity: usize,
    overflow: O,
    mut fib: F,
  ) -> FiberStreamRing<I, E>
  where
    O: Fn(I) -> Result<(), E>,
    F: Fiber<Input = (), Yield = Option<I>, Return = Result<Option<I>, E>>,
    O: Send + 'static,
    F: Send + 'static,
    I: Send + 'static,
    E: Send + 'static,
  {
    let (rx, mut tx) = channel(capacity);
    self.add(move || loop {
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

  /// Adds a new ring stream fiber. Overflows will be ignored.
  fn add_stream_ring_skip<F, I, E>(
    self,
    capacity: usize,
    fib: F,
  ) -> FiberStreamRing<I, E>
  where
    F: Fiber<Input = (), Yield = Option<I>, Return = Result<Option<I>, E>>,
    F: Send + 'static,
    I: Send + 'static,
    E: Send + 'static,
  {
    self.add_stream_ring(capacity, |_| Ok(()), fib)
  }

  /// Adds a new ring stream fiber. Overflows will overwrite.
  fn add_stream_ring_overwrite<F, I, E>(
    self,
    capacity: usize,
    mut fib: F,
  ) -> FiberStreamRing<I, E>
  where
    F: Fiber<Input = (), Yield = Option<I>, Return = Result<Option<I>, E>>,
    F: Send + 'static,
    I: Send + 'static,
    E: Send + 'static,
  {
    let (rx, mut tx) = channel(capacity);
    self.add(move || loop {
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
}

impl<T: ThrAttach, U: ThrToken<T>> ThrStreamRing<T> for U {}
