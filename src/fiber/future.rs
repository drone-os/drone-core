use core::mem;
use fiber::{spawn, Fiber, FiberState, YieldOption};
use sync::spsc::oneshot::{channel, Receiver, RecvError};
use thread::prelude::*;

/// A future for result from another thread.
///
/// This future can be created by the instance of [`Thread`](::thread::Thread).
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

  #[inline(always)]
  fn poll(&mut self) -> Poll<R, E> {
    self.rx.poll().map_err(|err| match err {
      RecvError::Complete(err) => err,
      RecvError::Canceled => unsafe { mem::unreachable() },
    })
  }
}

/// Spawns a new future fiber on the given `thread`.
pub fn spawn_future<T, U, F, Y, R, E>(
  thread: T,
  mut fiber: F,
) -> FiberFuture<R, E>
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
  spawn(thread, move || loop {
    if tx.is_canceled() {
      break;
    }
    match fiber.resume(()) {
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
