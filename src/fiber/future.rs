use core::mem;
use sync::spsc::oneshot::{channel, Receiver, RecvError};
use thread::prelude::*;

/// A future for result from another thread.
///
/// This future can be created by the instance of [`Thread`].
///
/// [`Thread`]: ../trait.Thread.html
#[must_use]
pub struct FiberFuture<R, E> {
  rx: Receiver<R, E>,
}

impl<R, E> FiberFuture<R, E> {
  pub(crate) fn new<T, G>(thread: &T, mut gen: G) -> Self
  where
    T: Thread,
    G: Generator<Yield = (), Return = Result<R, E>>,
    G: Send + 'static,
    R: Send + 'static,
    E: Send + 'static,
  {
    let (tx, rx) = channel();
    thread.fibers().add(move || loop {
      if tx.is_canceled() {
        break;
      }
      match gen.resume() {
        Yielded(()) => {}
        Complete(complete) => {
          tx.send(complete).ok();
          break;
        }
      }
      yield;
    });
    Self { rx }
  }

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
