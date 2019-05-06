use crate::{
  fib::{Fiber, FiberState, YieldNone},
  sync::spsc::oneshot::{channel, Receiver, RecvError},
  thr::prelude::*,
};
use core::{
  convert::identity,
  future::Future,
  intrinsics::unreachable,
  pin::Pin,
  task::{Context, Poll},
};

/// A future for a single value from another thread.
#[must_use]
pub struct FiberFuture<R> {
  rx: Receiver<R, !>,
}

/// A future for a single result from another thread.
#[must_use]
pub struct TryFiberFuture<R, E> {
  rx: Receiver<R, E>,
}

impl<R> FiberFuture<R> {
  /// Gracefully close this future, preventing sending any future messages.
  #[inline]
  pub fn close(&mut self) {
    self.rx.close()
  }
}

impl<R, E> TryFiberFuture<R, E> {
  /// Gracefully close this future, preventing sending any future messages.
  #[inline]
  pub fn close(&mut self) {
    self.rx.close()
  }
}

impl<R> Future for FiberFuture<R> {
  type Output = R;

  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<R> {
    let rx = unsafe { self.map_unchecked_mut(|x| &mut x.rx) };
    rx.poll(cx).map(|value| match value {
      Ok(value) => value,
      Err(RecvError::Canceled) => unsafe { unreachable() },
    })
  }
}

impl<R, E> Future for TryFiberFuture<R, E> {
  type Output = Result<R, E>;

  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<R, E>> {
    let rx = unsafe { self.map_unchecked_mut(|x| &mut x.rx) };
    rx.poll(cx).map_err(|err| match err {
      RecvError::Complete(err) => err,
      RecvError::Canceled => unsafe { unreachable() },
    })
  }
}

/// Future fiber extension to the thread token.
pub trait ThrFiberFuture<T: ThrAttach>: ThrToken<T> {
  /// Adds a new future fiber.
  fn add_future<F, Y, R>(self, fib: F) -> FiberFuture<R>
  where
    F: Fiber<Input = (), Yield = Y, Return = R>,
    Y: YieldNone,
    F: Send + 'static,
    R: Send + 'static,
  {
    FiberFuture {
      rx: add_rx(self, fib, Ok),
    }
  }

  /// Adds a new fallible future fiber.
  fn add_try_future<F, Y, R, E>(self, fib: F) -> TryFiberFuture<R, E>
  where
    F: Fiber<Input = (), Yield = Y, Return = Result<R, E>>,
    Y: YieldNone,
    F: Send + 'static,
    R: Send + 'static,
    E: Send + 'static,
  {
    TryFiberFuture {
      rx: add_rx(self, fib, identity),
    }
  }
}

#[inline]
fn add_rx<T, U, F, Y, R, E, C>(thr: T, mut fib: F, convert: C) -> Receiver<R, E>
where
  T: ThrToken<U>,
  U: ThrAttach,
  F: Fiber<Input = (), Yield = Y>,
  Y: YieldNone,
  C: FnOnce(F::Return) -> Result<R, E>,
  F: Send + 'static,
  R: Send + 'static,
  E: Send + 'static,
  C: Send + 'static,
{
  let (rx, tx) = channel();
  thr.add(move || {
    loop {
      if tx.is_canceled() {
        break;
      }
      match unsafe { Pin::new_unchecked(&mut fib) }.resume(()) {
        FiberState::Yielded(_) => {}
        FiberState::Complete(complete) => {
          tx.send(convert(complete)).ok();
          break;
        }
      }
      yield;
    }
  });
  rx
}

impl<T: ThrAttach, U: ThrToken<T>> ThrFiberFuture<T> for U {}
