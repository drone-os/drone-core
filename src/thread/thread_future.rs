use core::intrinsics;
use core::ops::Generator;
use core::ops::GeneratorState::*;
use futures::{Future, Poll};
use sync::spsc::oneshot::{channel, Receiver, RecvError};
use thread::Thread;

/// A future for result from another thread.
///
/// This future is created by the [`future`] method on [`Thread`]. See its
/// documentation for more.
///
/// [`Thread`]: ../trait.Thread.html
/// [`future`]: ../trait.Thread.html#method.future
#[must_use]
pub struct ThreadFuture<R, E> {
  rx: Receiver<R, E>,
}

impl<R, E> ThreadFuture<R, E> {
  #[inline(always)]
  pub(crate) fn new<T, G>(thread: &T, mut generator: G) -> Self
  where
    T: Thread,
    G: Generator<Yield = (), Return = Result<R, E>>,
    G: Send + 'static,
    R: Send + 'static,
    E: Send + 'static,
  {
    let (tx, rx) = channel();
    thread.routine(move || {
      loop {
        if tx.is_canceled() {
          break;
        }
        match generator.resume() {
          Yielded(()) => (),
          Complete(complete) => {
            tx.send(complete).ok();
            break;
          }
        }
        yield;
      }
    });
    Self { rx }
  }
}

impl<R, E> Future for ThreadFuture<R, E> {
  type Item = R;
  type Error = E;

  fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
    self.rx.poll().map_err(|err| match err {
      RecvError::Complete(err) => err,
      RecvError::Canceled => unsafe { intrinsics::unreachable() },
    })
  }
}
