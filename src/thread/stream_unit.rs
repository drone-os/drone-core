use core::ops::Generator;
use core::ops::GeneratorState::*;
use sync::spsc::unit::{channel, Receiver, SendError};
use thread::Thread;

pub(crate) fn stream_unit<T, G, E, O>(
  thread: &T,
  mut generator: G,
  overflow: O,
) -> Receiver<E>
where
  T: Thread,
  G: Generator<Yield = Option<()>, Return = Result<Option<()>, E>>,
  O: Fn() -> Result<(), E>,
  G: Send + 'static,
  E: Send + 'static,
  O: Send + 'static,
{
  let (mut tx, rx) = channel();
  thread.routine(move || {
    loop {
      if tx.is_canceled() {
        break;
      }
      match generator.resume() {
        Yielded(None) => {}
        Yielded(Some(())) => match tx.send() {
          Ok(()) => {}
          Err(SendError::Canceled) => {
            break;
          }
          Err(SendError::Overflow) => match overflow() {
            Ok(()) => {}
            Err(err) => {
              tx.send_err(err).ok();
              break;
            }
          },
        },
        Complete(Ok(None)) => {
          break;
        }
        Complete(Ok(Some(()))) => {
          tx.send().ok();
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
