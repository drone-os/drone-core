use core::ops::Generator;
use core::ops::GeneratorState::*;
use sync::spsc::ring::{channel, Receiver, SendError, SendErrorKind};
use thread::Thread;

pub(crate) fn stream_ring<T, G, I, E, O>(
  thread: &T,
  capacity: usize,
  mut generator: G,
  overflow: O,
) -> Receiver<I, E>
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
  thread.routine(move || {
    loop {
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
    }
  });
  rx
}

pub(crate) fn stream_ring_overwrite<T, G, I, E>(
  thread: &T,
  capacity: usize,
  mut generator: G,
) -> Receiver<I, E>
where
  T: Thread,
  G: Generator<Yield = Option<I>, Return = Result<Option<I>, E>>,
  G: Send + 'static,
  I: Send + 'static,
  E: Send + 'static,
{
  let (mut tx, rx) = channel(capacity);
  thread.routine(move || {
    loop {
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
    }
  });
  rx
}
