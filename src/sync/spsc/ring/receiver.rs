use super::{Inner, COMPLETE, INDEX_BITS, INDEX_MASK, RX_LOCK};
use alloc::sync::Arc;
use core::ptr;
use core::sync::atomic::Ordering::*;
use futures::prelude::*;
use sync::spsc::SpscInner;

/// The receiving-half of [`ring::channel`](channel).
#[must_use]
pub struct Receiver<T, E> {
  inner: Arc<Inner<T, E>>,
}

impl<T, E> Receiver<T, E> {
  #[inline(always)]
  pub(super) fn new(inner: Arc<Inner<T, E>>) -> Self {
    Self { inner }
  }

  /// Gracefully close this `Receiver`, preventing sending any future
  /// messages.
  #[inline(always)]
  pub fn close(&mut self) {
    self.inner.close_rx()
  }
}

impl<T, E> Stream for Receiver<T, E> {
  type Item = T;
  type Error = E;

  #[inline(always)]
  fn poll_next(&mut self, cx: &mut task::Context) -> Poll<Option<T>, E> {
    self.inner.recv(cx)
  }
}

impl<T, E> Drop for Receiver<T, E> {
  #[inline(always)]
  fn drop(&mut self) {
    self.inner.drop_rx();
  }
}

impl<T, E> Inner<T, E> {
  fn recv(&self, cx: &mut task::Context) -> Poll<Option<T>, E> {
    let some_value = || {
      |index| unsafe {
        Async::Ready(Some(ptr::read(self.buffer.ptr().add(index))))
      }
    };
    self
      .update(self.state_load(Acquire), Acquire, Acquire, |state| {
        if let Some(index) = Self::take_index(state, self.buffer.cap()) {
          Ok(Ok(index))
        } else if *state & COMPLETE == 0 {
          *state |= RX_LOCK;
          Ok(Err(*state))
        } else {
          Err(())
        }
      })
      .and_then(|state| {
        state.map(some_value()).or_else(|state| {
          unsafe {
            (*self.rx_waker.get()).get_or_insert_with(|| cx.waker().clone());
          }
          self
            .update(state, AcqRel, Relaxed, |state| {
              *state ^= RX_LOCK;
              if let Some(index) = Self::take_index(state, self.buffer.cap()) {
                Ok(Ok(index))
              } else {
                Ok(Err(*state))
              }
            })
            .and_then(|state| {
              state.map(some_value()).or_else(|state| {
                if state & COMPLETE == 0 {
                  Ok(Async::Pending)
                } else {
                  Err(())
                }
              })
            })
        })
      })
      .or_else(|()| {
        let err = unsafe { &mut *self.err.get() };
        err.take().map_or_else(|| Ok(Async::Ready(None)), Err)
      })
  }

  #[inline(always)]
  pub(super) fn take_index(
    state: &mut usize,
    capacity: usize,
  ) -> Option<usize> {
    let count = *state & INDEX_MASK;
    if count == 0 {
      return None;
    }
    let begin = *state >> INDEX_BITS & INDEX_MASK;
    *state >>= INDEX_BITS << 1;
    *state <<= INDEX_BITS;
    *state |= begin.wrapping_add(1).wrapping_rem(capacity);
    *state <<= INDEX_BITS;
    *state |= count.wrapping_sub(1);
    Some(begin)
  }
}
