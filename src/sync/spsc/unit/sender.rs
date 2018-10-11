use super::{Inner, COMPLETE, LOCK_BITS, LOCK_MASK, RX_LOCK};
use alloc::sync::Arc;
use core::sync::atomic::Ordering::*;
use futures::prelude::*;
use futures::task::Waker;
use sync::spsc::SpscInner;

/// The sending-half of [`unit::channel`](channel).
pub struct Sender<E> {
  inner: Arc<Inner<E>>,
}

/// Error returned from [`Sender::send`](Sender::send).
#[derive(Debug, Fail)]
pub enum SendError {
  /// The corresponding [`Receiver`](Receiver) is dropped.
  #[fail(display = "Receiver is dropped.")]
  Canceled,
  /// Counter overflow.
  #[fail(display = "Channel buffer overflow.")]
  Overflow,
}

impl<E> Sender<E> {
  #[inline(always)]
  pub(super) fn new(inner: Arc<Inner<E>>) -> Self {
    Self { inner }
  }

  /// Sends a unit across the channel.
  #[inline(always)]
  pub fn send(&mut self) -> Result<(), SendError> {
    self.inner.send()
  }

  /// Completes this stream with an error.
  ///
  /// If the value is successfully enqueued, then `Ok(())` is returned. If the
  /// receiving end was dropped before this function was called, then `Err` is
  /// returned with the value provided.
  #[inline(always)]
  pub fn send_err(self, err: E) -> Result<(), E> {
    self.inner.send_err(err)
  }

  /// Polls this [`Sender`] half to detect whether the [`Receiver`] this has
  /// paired with has gone away.
  ///
  /// # Panics
  ///
  /// Like `Future::poll`, this function will panic if it's not called from
  /// within the context of a task. In other words, this should only ever be
  /// called from inside another future.
  ///
  /// If you're calling this function from a context that does not have a task,
  /// then you can use the [`is_canceled`] API instead.
  ///
  /// [`Sender`]: Sender
  /// [`Receiver`]: super::Receiver
  /// [`is_canceled`]: Sender::is_canceled
  #[inline(always)]
  pub fn poll_cancel(&mut self, cx: &mut task::Context) -> Poll<(), ()> {
    self.inner.poll_cancel(cx)
  }

  /// Tests to see whether this [`Sender`]'s corresponding [`Receiver`] has gone
  /// away.
  ///
  /// [`Sender`]: Sender
  /// [`Receiver`]: super::Receiver
  #[inline(always)]
  pub fn is_canceled(&self) -> bool {
    self.inner.is_canceled()
  }
}

impl<E> Drop for Sender<E> {
  #[inline(always)]
  fn drop(&mut self) {
    self.inner.drop_tx();
  }
}

impl<E> Inner<E> {
  fn send(&self) -> Result<(), SendError> {
    self
      .update(self.state_load(Relaxed), Acquire, Relaxed, |state| {
        let mut lock = *state & LOCK_MASK;
        if lock & COMPLETE != 0 {
          return Err(SendError::Canceled);
        }
        *state = (*state as isize >> LOCK_BITS) as usize;
        *state = state.wrapping_add(1);
        if *state == 0 {
          return Err(SendError::Overflow);
        }
        let rx_locked = if lock & RX_LOCK == 0 {
          lock |= RX_LOCK;
          true
        } else {
          false
        };
        *state <<= LOCK_BITS;
        *state |= lock;
        if rx_locked {
          Ok(Some(*state))
        } else {
          Ok(None)
        }
      })
      .map(|state| {
        state.map(|state| {
          unsafe {
            (*self.rx_waker.get()).as_ref().map(Waker::wake);
          }
          self.update(state, Release, Relaxed, |state| {
            *state ^= RX_LOCK;
            Ok::<(), ()>(())
          })
        });
      })
  }

  fn send_err(&self, err: E) -> Result<(), E> {
    if self.is_canceled() {
      Err(err)
    } else {
      unsafe { *self.err.get() = Some(err) };
      Ok(())
    }
  }
}
