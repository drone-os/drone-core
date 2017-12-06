use core::mem;
use sync::spsc::oneshot::{channel, Receiver, RecvError};

/// A future for result from another thread.
///
/// This future is created by the [`future`] method on [`Thread`]. See its
/// documentation for more.
///
/// [`Thread`]: ../trait.Thread.html
/// [`future`]: ../trait.Thread.html#method.future
#[must_use]
pub struct RoutineFuture<R, E> {
  rx: Receiver<R, E>,
}

impl<R, E> RoutineFuture<R, E> {
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
          Yielded(()) => {}
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

impl<R, E> Future for RoutineFuture<R, E> {
  type Item = R;
  type Error = E;

  fn poll(&mut self) -> Poll<R, E> {
    self.rx.poll().map_err(|err| match err {
      RecvError::Complete(err) => err,
      RecvError::Canceled => unsafe { mem::unreachable() },
    })
  }
}
