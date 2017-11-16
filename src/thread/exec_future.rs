use core::intrinsics;
use futures::{Async, Future, Poll};
use sync::spsc::oneshot::{channel, Receiver, RecvError};
use thread::{Executor, Thread};

/// A future for result from another thread future executor.
///
/// This future is created by the [`exec_future`] method on [`Thread`]. See its
/// documentation for more.
///
/// [`Thread`]: ../trait.Thread.html
/// [`exec_future`]: ../trait.Thread.html#method.exec_future
#[must_use]
pub struct ExecFuture<R, E> {
  rx: Receiver<R, E>,
}

impl<R, E> ExecFuture<R, E> {
  #[inline(always)]
  pub(crate) fn new<T, F>(thread: &T, future: F) -> Self
  where
    T: Thread,
    F: Future<Item = R, Error = E>,
    F: Send + 'static,
    R: Send + 'static,
    E: Send + 'static,
  {
    let (tx, rx) = channel();
    let mut executor = Executor::new(future);
    thread.routine(move || {
      loop {
        if tx.is_canceled() {
          break;
        }
        match executor.poll() {
          Ok(Async::NotReady) => (),
          Ok(Async::Ready(ready)) => {
            tx.send(Ok(ready)).ok();
            break;
          }
          Err(err) => {
            tx.send(Err(err)).ok();
            break;
          }
        }
        yield;
      }
    });
    Self { rx }
  }
}

impl<R, E> Future for ExecFuture<R, E> {
  type Item = R;
  type Error = E;

  fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
    self.rx.poll().map_err(|err| match err {
      RecvError::Complete(err) => err,
      RecvError::Canceled => unsafe { intrinsics::unreachable() },
    })
  }
}
