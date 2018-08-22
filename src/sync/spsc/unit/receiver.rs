use super::{Inner, COMPLETE, LOCK_BITS, LOCK_MASK, RX_LOCK};
use alloc::sync::Arc;
use core::sync::atomic::Ordering::*;
use futures::prelude::*;
use sync::spsc::SpscInner;

/// The receiving-half of [`unit::channel`](channel).
#[must_use]
pub struct Receiver<E> {
  inner: Arc<Inner<E>>,
}

impl<E> Receiver<E> {
  #[inline(always)]
  pub(super) fn new(inner: Arc<Inner<E>>) -> Self {
    Self { inner }
  }

  /// Gracefully close this `Receiver`, preventing sending any future
  /// messages.
  #[inline(always)]
  pub fn close(&mut self) {
    self.inner.close_rx()
  }
}

impl<E> Stream for Receiver<E> {
  type Item = ();
  type Error = E;

  #[inline(always)]
  fn poll_next(&mut self, cx: &mut task::Context) -> Poll<Option<()>, E> {
    self.inner.recv(cx)
  }
}

impl<E> Drop for Receiver<E> {
  #[inline(always)]
  fn drop(&mut self) {
    self.inner.drop_rx();
  }
}

impl<E> Inner<E> {
  fn recv(&self, cx: &mut task::Context) -> Poll<Option<()>, E> {
    let some_unit = || || Ok(Async::Ready(Some(())));
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
      }).and_then(|state| {
        state.map_or_else(some_unit(), |state| {
          unsafe {
            (*self.rx_waker.get()).get_or_insert_with(|| cx.waker().clone());
          }
          self
            .update(state, AcqRel, Relaxed, |state| {
              *state ^= RX_LOCK;
              if Self::take(state) {
                Ok(None)
              } else {
                Ok(Some(*state))
              }
            }).and_then(|state| {
              state.map_or_else(some_unit(), |state| {
                if state & COMPLETE == 0 {
                  Ok(Async::Pending)
                } else {
                  Err(())
                }
              })
            })
        })
      }).or_else(|()| {
        let err = unsafe { &mut *self.err.get() };
        err.take().map_or_else(|| Ok(Async::Ready(None)), Err)
      })
  }

  #[inline(always)]
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
