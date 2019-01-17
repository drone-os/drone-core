use super::{Inner, COMPLETE, INDEX_BITS, INDEX_MASK, RX_LOCK};
use crate::sync::spsc::SpscInner;
use alloc::sync::Arc;
use core::{
  pin::Pin,
  ptr,
  sync::atomic::Ordering::*,
  task::{LocalWaker, Poll},
};
use futures::stream::Stream;

/// The receiving-half of [`ring::channel`](super::channel).
#[must_use]
pub struct Receiver<T, E> {
  inner: Arc<Inner<T, E>>,
}

impl<T, E> Receiver<T, E> {
  #[inline]
  pub(super) fn new(inner: Arc<Inner<T, E>>) -> Self {
    Self { inner }
  }

  /// Gracefully close this `Receiver`, preventing sending any future
  /// messages.
  #[inline]
  pub fn close(&mut self) {
    self.inner.close_rx()
  }
}

impl<T, E> Stream for Receiver<T, E> {
  type Item = Result<T, E>;

  #[inline]
  fn poll_next(
    self: Pin<&mut Self>,
    lw: &LocalWaker,
  ) -> Poll<Option<Self::Item>> {
    self.inner.recv(lw)
  }
}

impl<T, E> Drop for Receiver<T, E> {
  #[inline]
  fn drop(&mut self) {
    self.inner.drop_rx();
  }
}

impl<T, E> Inner<T, E> {
  fn recv(&self, lw: &LocalWaker) -> Poll<Option<Result<T, E>>> {
    let some_value = |index| unsafe {
      Poll::Ready(Some(Ok(ptr::read(self.buffer.ptr().add(index)))))
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
        state.map(some_value).or_else(|state| {
          unsafe {
            (*self.rx_waker.get())
              .get_or_insert_with(|| lw.clone().into_waker());
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
              state.map(some_value).or_else(|state| {
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
