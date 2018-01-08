use sync::spsc::ring::{channel, Receiver, SendError, SendErrorKind};
use thread::prelude::*;

/// A stream of results from another thread.
///
/// This stream can be created by the instance of [`Thread`].
///
/// [`Thread`]: ../trait.Thread.html
#[must_use]
pub struct RoutineStreamRing<I, E> {
  rx: Receiver<I, E>,
}

impl<I, E> RoutineStreamRing<I, E> {
  pub(crate) fn new<T, G, O>(
    thread: &T,
    capacity: usize,
    mut generator: G,
    overflow: O,
  ) -> Self
  where
    T: Thread,
    G: Generator<Yield = Option<I>, Return = Result<Option<I>, E>>,
    O: Fn(I) -> Result<(), E>,
    G: Send + 'static,
    I: Send + 'static,
    E: Send + 'static,
    O: Send + 'static,
  {
    let (mut tx, rx) = channel(capacity);
    thread.routines().push(move || loop {
      if tx.is_canceled() {
        break;
      }
      match generator.resume() {
        Yielded(None) => {}
        Yielded(Some(value)) => match tx.send(value) {
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
        Complete(Ok(None)) => {
          break;
        }
        Complete(Ok(Some(value))) => {
          tx.send(value).ok();
          break;
        }
        Complete(Err(err)) => {
          tx.send_err(err).ok();
          break;
        }
      }
      yield;
    });
    Self { rx }
  }

  pub(crate) fn new_overwrite<T, G>(
    thread: &T,
    capacity: usize,
    mut generator: G,
  ) -> Self
  where
    T: Thread,
    G: Generator<Yield = Option<I>, Return = Result<Option<I>, E>>,
    G: Send + 'static,
    I: Send + 'static,
    E: Send + 'static,
  {
    let (mut tx, rx) = channel(capacity);
    thread.routines().push(move || loop {
      if tx.is_canceled() {
        break;
      }
      match generator.resume() {
        Yielded(None) => {}
        Yielded(Some(value)) => match tx.send_overwrite(value) {
          Ok(()) => (),
          Err(_) => break,
        },
        Complete(Ok(None)) => {
          break;
        }
        Complete(Ok(Some(value))) => {
          tx.send_overwrite(value).ok();
          break;
        }
        Complete(Err(err)) => {
          tx.send_err(err).ok();
          break;
        }
      }
      yield;
    });
    Self { rx }
  }

  /// Gracefully close this stream, preventing sending any future messages.
  #[inline(always)]
  pub fn close(&mut self) {
    self.rx.close()
  }
}

impl<I, E> Stream for RoutineStreamRing<I, E> {
  type Item = I;
  type Error = E;

  #[inline(always)]
  fn poll(&mut self) -> Poll<Option<I>, E> {
    self.rx.poll()
  }
}
