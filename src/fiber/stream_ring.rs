use fiber::{spawn, Fiber, FiberState};
use sync::spsc::ring::{channel, Receiver, SendError, SendErrorKind};
use thread::prelude::*;

/// A stream of results from another thread.
///
/// This stream can be created by the instance of [`Thread`](::thread::Thread).
#[must_use]
pub struct FiberStreamRing<I, E> {
  rx: Receiver<I, E>,
}

impl<I, E> FiberStreamRing<I, E> {
  /// Gracefully close this stream, preventing sending any future messages.
  #[inline(always)]
  pub fn close(&mut self) {
    self.rx.close()
  }
}

impl<I, E> Stream for FiberStreamRing<I, E> {
  type Item = I;
  type Error = E;

  #[inline(always)]
  fn poll(&mut self) -> Poll<Option<I>, E> {
    self.rx.poll()
  }
}

/// Spawns a new ring stream fiber on the given `thread`.
pub fn spawn_stream_ring<T, U, O, F, I, E>(
  thread: T,
  capacity: usize,
  overflow: O,
  mut fiber: F,
) -> FiberStreamRing<I, E>
where
  T: AsRef<U>,
  U: Thread,
  O: Fn(I) -> Result<(), E>,
  F: Fiber<Input = (), Yield = Option<I>, Return = Result<Option<I>, E>>,
  O: Send + 'static,
  F: Send + 'static,
  I: Send + 'static,
  E: Send + 'static,
{
  let (rx, mut tx) = channel(capacity);
  spawn(thread, move || loop {
    if tx.is_canceled() {
      break;
    }
    match fiber.resume(()) {
      FiberState::Yielded(None) => {}
      FiberState::Yielded(Some(value)) => match tx.send(value) {
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
      FiberState::Complete(Ok(None)) => {
        break;
      }
      FiberState::Complete(Ok(Some(value))) => {
        tx.send(value).ok();
        break;
      }
      FiberState::Complete(Err(err)) => {
        tx.send_err(err).ok();
        break;
      }
    }
    yield;
  });
  FiberStreamRing { rx }
}

/// Spawns a new ring stream fiber on the given `thread`. Overflows will be
/// ignored.
#[inline(always)]
pub fn spawn_stream_ring_skip<T, U, F, I, E>(
  thread: T,
  capacity: usize,
  fiber: F,
) -> FiberStreamRing<I, E>
where
  T: AsRef<U>,
  U: Thread,
  F: Fiber<Input = (), Yield = Option<I>, Return = Result<Option<I>, E>>,
  F: Send + 'static,
  I: Send + 'static,
  E: Send + 'static,
{
  spawn_stream_ring(thread, capacity, |_| Ok(()), fiber)
}

/// Spawns a new ring stream fiber on the given `thread`. Overflows will
/// overwrite.
pub fn spawn_stream_ring_overwrite<T, U, F, I, E>(
  thread: T,
  capacity: usize,
  mut fiber: F,
) -> FiberStreamRing<I, E>
where
  T: AsRef<U>,
  U: Thread,
  F: Fiber<Input = (), Yield = Option<I>, Return = Result<Option<I>, E>>,
  F: Send + 'static,
  I: Send + 'static,
  E: Send + 'static,
{
  let (rx, mut tx) = channel(capacity);
  spawn(thread, move || loop {
    if tx.is_canceled() {
      break;
    }
    match fiber.resume(()) {
      FiberState::Yielded(None) => {}
      FiberState::Yielded(Some(value)) => match tx.send_overwrite(value) {
        Ok(()) => (),
        Err(_) => break,
      },
      FiberState::Complete(Ok(None)) => {
        break;
      }
      FiberState::Complete(Ok(Some(value))) => {
        tx.send_overwrite(value).ok();
        break;
      }
      FiberState::Complete(Err(err)) => {
        tx.send_err(err).ok();
        break;
      }
    }
    yield;
  });
  FiberStreamRing { rx }
}
