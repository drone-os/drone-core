use core::mem;
use fib::{self, Fiber, FiberState, YieldOption};
use sync::spsc::oneshot::{channel, Receiver, RecvError};
use thr::prelude::*;

/// A future for result from another thread.
///
/// This future can be created by the instance of [`Thread`](::thr::Thread).
#[must_use]
pub struct FiberFuture<R, E> {
  rx: Receiver<R, E>,
}

impl<R, E> FiberFuture<R, E> {
  /// Gracefully close this future, preventing sending any future messages.
  #[inline(always)]
  pub fn close(&mut self) {
    self.rx.close()
  }
}

impl<R, E> Future for FiberFuture<R, E> {
  type Item = R;
  type Error = E;

  fn poll(&mut self) -> Poll<R, E> {
    self.rx.poll().map_err(|err| match err {
      RecvError::Complete(err) => err,
      RecvError::Canceled => unsafe { mem::unreachable() },
    })
  }
}

/// Adds a new future fiber on the given `thr`.
pub fn add_future<T, U, F, Y, R, E>(thr: T, mut fib: F) -> FiberFuture<R, E>
where
  T: AsRef<U>,
  U: Thread,
  F: Fiber<Input = (), Yield = Y, Return = Result<R, E>>,
  F: Send + 'static,
  Y: YieldOption,
  R: Send + 'static,
  E: Send + 'static,
{
  let (rx, tx) = channel();
  fib::add(thr, move || loop {
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
