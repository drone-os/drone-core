use crate::{
  fib::{Fiber, FiberState, YieldNone},
  sync::spsc::oneshot::{channel, Receiver, RecvError},
  thr::prelude::*,
};
use core::intrinsics::unreachable;
use futures::prelude::*;

/// A future for result from another thread.
///
/// This future can be created by the instance of [`Thread`](::thr::Thread).
#[must_use]
pub struct FiberFuture<R, E> {
  rx: Receiver<R, E>,
}

impl<R, E> FiberFuture<R, E> {
  /// Gracefully close this future, preventing sending any future messages.
  #[inline]
  pub fn close(&mut self) {
    self.rx.close()
  }
}

impl<R, E> Future for FiberFuture<R, E> {
  type Item = R;
  type Error = E;

  fn poll(&mut self, cx: &mut task::Context) -> Poll<R, E> {
    self.rx.poll(cx).map_err(|err| match err {
      RecvError::Complete(err) => err,
      RecvError::Canceled => unsafe { unreachable() },
    })
  }
}

/// Future fiber extension to the thread token.
pub trait ThrFiberFuture<T: ThrAttach>: ThrToken<T> {
  /// Adds a new future fiber.
  fn add_future<F, Y, R, E>(self, mut fib: F) -> FiberFuture<R, E>
  where
    F: Fiber<Input = (), Yield = Y, Return = Result<R, E>>,
    F: Send + 'static,
    Y: YieldNone,
    R: Send + 'static,
    E: Send + 'static,
  {
    let (rx, tx) = channel();
    self.add(move || loop {
      if tx.is_canceled() {
        break;
      }
      match fib.resume(()) {
        FiberState::Yielded(_) => {}
        FiberState::Complete(complete) => {
          tx.send(complete).ok();
          break;
        }
      }
      yield;
    });
    FiberFuture { rx }
  }
}

impl<T: ThrAttach, U: ThrToken<T>> ThrFiberFuture<T> for U {}
