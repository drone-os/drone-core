use crate::{
  fib::{Fiber, FiberState},
  sync::spsc::unit::{channel, Receiver, SendError},
  thr::prelude::*,
};
use core::{
  convert::identity,
  pin::Pin,
  task::{Context, Poll},
};
use futures::Stream;

/// A stream of values from another thread.
#[must_use]
pub struct FiberStreamUnit {
  rx: Receiver<!>,
}

/// A stream of results from another thread.
#[must_use]
pub struct TryFiberStreamUnit<E> {
  rx: Receiver<E>,
}

impl FiberStreamUnit {
  /// Gracefully close this stream, preventing sending any future messages.
  #[inline]
  pub fn close(&mut self) {
    self.rx.close()
  }
}

impl<E> TryFiberStreamUnit<E> {
  /// Gracefully close this stream, preventing sending any future messages.
  #[inline]
  pub fn close(&mut self) {
    self.rx.close()
  }
}

impl Stream for FiberStreamUnit {
  type Item = ();

  #[inline]
  fn poll_next(
    self: Pin<&mut Self>,
    cx: &mut Context<'_>,
  ) -> Poll<Option<Self::Item>> {
    let rx = unsafe { self.map_unchecked_mut(|x| &mut x.rx) };
    rx.poll_next(cx).map(|value| {
      value.map(|value| match value {
        Ok(value) => value,
      })
    })
  }
}

impl<E> Stream for TryFiberStreamUnit<E> {
  type Item = Result<(), E>;

  #[inline]
  fn poll_next(
    self: Pin<&mut Self>,
    cx: &mut Context<'_>,
  ) -> Poll<Option<Self::Item>> {
    let rx = unsafe { self.map_unchecked_mut(|x| &mut x.rx) };
    rx.poll_next(cx)
  }
}

/// Unit stream extension to the thread token.
pub trait ThrStreamUnit<T: ThrAttach>: ThrToken<T> {
  /// Adds a new unit stream fiber. Overflows will be ignored.
  fn add_stream_skip<F>(self, fib: F) -> FiberStreamUnit
  where
    F: Fiber<Input = (), Yield = Option<()>, Return = Option<()>>,
    F: Send + 'static,
  {
    FiberStreamUnit {
      rx: add_rx(self, || Ok(()), fib, Ok),
    }
  }

  /// Adds a new fallible unit stream fiber.
  fn add_stream<O, F, E>(self, overflow: O, fib: F) -> TryFiberStreamUnit<E>
  where
    O: Fn() -> Result<(), E>,
    F: Fiber<Input = (), Yield = Option<()>, Return = Result<Option<()>, E>>,
    O: Send + 'static,
    F: Send + 'static,
    E: Send + 'static,
  {
    TryFiberStreamUnit {
      rx: add_rx(self, overflow, fib, identity),
    }
  }
}

#[inline]
fn add_rx<T, U, O, F, E, C>(
  thr: T,
  overflow: O,
  mut fib: F,
  convert: C,
) -> Receiver<E>
where
  T: ThrToken<U>,
  U: ThrAttach,
  O: Fn() -> Result<(), E>,
  F: Fiber<Input = (), Yield = Option<()>>,
  C: FnOnce(F::Return) -> Result<Option<()>, E>,
  O: Send + 'static,
  F: Send + 'static,
  E: Send + 'static,
  C: Send + 'static,
{
  let (rx, mut tx) = channel();
  thr.add(move || {
    loop {
      if tx.is_canceled() {
        break;
      }
      match unsafe { Pin::new_unchecked(&mut fib) }.resume(()) {
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
        FiberState::Complete(value) => {
          match convert(value) {
            Ok(None) => {}
            Ok(Some(())) => {
              tx.send().ok();
            }
            Err(err) => {
              tx.send_err(err).ok();
            }
          }
          break;
        }
      }
      yield;
    }
  });
  rx
}

impl<T: ThrAttach, U: ThrToken<T>> ThrStreamUnit<T> for U {}
