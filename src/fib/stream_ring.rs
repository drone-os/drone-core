use crate::{
  fib::{Fiber, FiberState},
  sync::spsc::ring::{channel, Receiver, SendError, SendErrorKind},
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
pub struct FiberStreamRing<I> {
  rx: Receiver<I, !>,
}

/// A stream of results from another thread.
#[must_use]
pub struct TryFiberStreamRing<I, E> {
  rx: Receiver<I, E>,
}

impl<I> FiberStreamRing<I> {
  /// Gracefully close this stream, preventing sending any future messages.
  #[inline]
  pub fn close(&mut self) {
    self.rx.close()
  }
}

impl<I, E> TryFiberStreamRing<I, E> {
  /// Gracefully close this stream, preventing sending any future messages.
  #[inline]
  pub fn close(&mut self) {
    self.rx.close()
  }
}

impl<I> Stream for FiberStreamRing<I> {
  type Item = I;

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

impl<I, E> Stream for TryFiberStreamRing<I, E> {
  type Item = Result<I, E>;

  #[inline]
  fn poll_next(
    self: Pin<&mut Self>,
    cx: &mut Context<'_>,
  ) -> Poll<Option<Self::Item>> {
    let rx = unsafe { self.map_unchecked_mut(|x| &mut x.rx) };
    rx.poll_next(cx)
  }
}

/// Ring stream extension to the thread token.
pub trait ThrStreamRing<T: ThrAttach>: ThrToken<T> {
  /// Adds a new ring stream fiber. Overflows will be ignored.
  fn add_stream_ring_skip<F, I>(
    self,
    capacity: usize,
    fib: F,
  ) -> FiberStreamRing<I>
  where
    F: Fiber<Input = (), Yield = Option<I>, Return = Option<I>>,
    F: Send + 'static,
    I: Send + 'static,
  {
    FiberStreamRing {
      rx: add_rx(self, capacity, |_| Ok(()), fib, Ok),
    }
  }

  /// Adds a new ring stream fiber. Overflows will overwrite.
  fn add_stream_ring_overwrite<F, I>(
    self,
    capacity: usize,
    fib: F,
  ) -> FiberStreamRing<I>
  where
    F: Fiber<Input = (), Yield = Option<I>, Return = Option<I>>,
    F: Send + 'static,
    I: Send + 'static,
  {
    FiberStreamRing {
      rx: add_rx_overwrite(self, capacity, fib, Ok),
    }
  }

  /// Adds a new fallible ring stream fiber.
  fn add_stream_ring<O, F, I, E>(
    self,
    capacity: usize,
    overflow: O,
    fib: F,
  ) -> TryFiberStreamRing<I, E>
  where
    O: Fn(I) -> Result<(), E>,
    F: Fiber<Input = (), Yield = Option<I>, Return = Result<Option<I>, E>>,
    O: Send + 'static,
    F: Send + 'static,
    I: Send + 'static,
    E: Send + 'static,
  {
    TryFiberStreamRing {
      rx: add_rx(self, capacity, overflow, fib, identity),
    }
  }

  /// Adds a new fallible ring stream fiber. Overflows will overwrite.
  fn add_try_stream_ring_overwrite<F, I, E>(
    self,
    capacity: usize,
    fib: F,
  ) -> TryFiberStreamRing<I, E>
  where
    F: Fiber<Input = (), Yield = Option<I>, Return = Result<Option<I>, E>>,
    F: Send + 'static,
    I: Send + 'static,
    E: Send + 'static,
  {
    TryFiberStreamRing {
      rx: add_rx_overwrite(self, capacity, fib, identity),
    }
  }
}

#[inline]
fn add_rx<T, U, O, F, I, E, C>(
  thr: T,
  capacity: usize,
  overflow: O,
  mut fib: F,
  convert: C,
) -> Receiver<I, E>
where
  T: ThrToken<U>,
  U: ThrAttach,
  O: Fn(I) -> Result<(), E>,
  F: Fiber<Input = (), Yield = Option<I>>,
  C: FnOnce(F::Return) -> Result<Option<I>, E>,
  O: Send + 'static,
  F: Send + 'static,
  I: Send + 'static,
  E: Send + 'static,
  C: Send + 'static,
{
  let (rx, mut tx) = channel(capacity);
  thr.add(move || {
    loop {
      if tx.is_canceled() {
        break;
      }
      match unsafe { Pin::new_unchecked(&mut fib) }.resume(()) {
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
        FiberState::Complete(value) => {
          match convert(value) {
            Ok(None) => {}
            Ok(Some(value)) => {
              tx.send(value).ok();
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

#[inline]
fn add_rx_overwrite<T, U, F, I, E, C>(
  thr: T,
  capacity: usize,
  mut fib: F,
  convert: C,
) -> Receiver<I, E>
where
  T: ThrToken<U>,
  U: ThrAttach,
  F: Fiber<Input = (), Yield = Option<I>>,
  C: FnOnce(F::Return) -> Result<Option<I>, E>,
  F: Send + 'static,
  I: Send + 'static,
  E: Send + 'static,
  C: Send + 'static,
{
  let (rx, mut tx) = channel(capacity);
  thr.add(move || {
    loop {
      if tx.is_canceled() {
        break;
      }
      match unsafe { Pin::new_unchecked(&mut fib) }.resume(()) {
        FiberState::Yielded(None) => {}
        FiberState::Yielded(Some(value)) => match tx.send_overwrite(value) {
          Ok(()) => (),
          Err(_) => break,
        },
        FiberState::Complete(value) => {
          match convert(value) {
            Ok(None) => {}
            Ok(Some(value)) => {
              tx.send_overwrite(value).ok();
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

impl<T: ThrAttach, U: ThrToken<T>> ThrStreamRing<T> for U {}
