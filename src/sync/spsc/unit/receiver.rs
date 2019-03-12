use super::{Inner, COMPLETE, LOCK_BITS, LOCK_MASK, RX_LOCK};
use crate::sync::spsc::SpscInner;
use alloc::sync::Arc;
use core::{
  pin::Pin,
  sync::atomic::Ordering::*,
  task::{Poll, Waker},
};
use futures::stream::Stream;

/// The receiving-half of [`unit::channel`](super::channel).
#[must_use]
pub struct Receiver<E> {
  inner: Arc<Inner<E>>,
}

impl<E> Receiver<E> {
  #[inline]
  pub(super) fn new(inner: Arc<Inner<E>>) -> Self {
    Self { inner }
  }

  /// Gracefully close this `Receiver`, preventing sending any future
  /// messages.
  #[inline]
  pub fn close(&mut self) {
    self.inner.close_rx()
  }
}

impl<E> Stream for Receiver<E> {
  type Item = Result<(), E>;

  #[inline]
  fn poll_next(
    self: Pin<&mut Self>,
    waker: &Waker,
  ) -> Poll<Option<Self::Item>> {
    self.inner.recv(waker)
  }
}

impl<E> Drop for Receiver<E> {
  #[inline]
  fn drop(&mut self) {
    self.inner.drop_rx();
  }
}

impl<E> Inner<E> {
  fn recv(&self, waker: &Waker) -> Poll<Option<Result<(), E>>> {
    let some_unit = || Ok(Poll::Ready(Some(Ok(()))));
    self
      .update(self.state_load(Acquire), Acquire, Acquire, |state| {
        if Self::take(state) {
          Ok(None)
        } else if *state & COMPLETE == 0 {
          *state |= RX_LOCK;
          Ok(Some(*state))
        } else {
          Err(())
        }
      })
      .and_then(|state| {
        state.map_or_else(some_unit, |state| {
          unsafe {
            (*self.rx_waker.get()).get_or_insert_with(|| waker.clone());
          }
          self
            .update(state, AcqRel, Relaxed, |state| {
              *state ^= RX_LOCK;
              if Self::take(state) {
                Ok(None)
              } else {
                Ok(Some(*state))
              }
            })
            .and_then(|state| {
              state.map_or_else(some_unit, |state| {
                if state & COMPLETE == 0 {
                  Ok(Poll::Pending)
                } else {
                  Err(())
                }
              })
            })
        })
      })
      .unwrap_or_else(|()| {
        Poll::Ready(unsafe { &mut *self.err.get() }.take().map(Err))
      })
  }

  #[inline]
  fn take(state: &mut usize) -> bool {
    let lock = *state & LOCK_MASK;
    *state >>= LOCK_BITS;
    let took = if *state == 0 {
      false
    } else {
      *state = state.wrapping_sub(1);
      true
    };
    *state <<= LOCK_BITS;
    *state |= lock;
    took
  }
}
