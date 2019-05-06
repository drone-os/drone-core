use super::{Inner, COMPLETE, INDEX_BITS, INDEX_MASK};
use crate::sync::spsc::{SpscInner, SpscInnerErr};
use alloc::sync::Arc;
use core::{
  pin::Pin,
  ptr,
  sync::atomic::Ordering,
  task::{Context, Poll},
};
use futures::stream::Stream;

const IS_TX_HALF: bool = false;

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
    self.inner.close_half(IS_TX_HALF)
  }
}

impl<T, E> Stream for Receiver<T, E> {
  type Item = Result<T, E>;

  #[inline]
  fn poll_next(
    self: Pin<&mut Self>,
    cx: &mut Context<'_>,
  ) -> Poll<Option<Self::Item>> {
    self.inner.poll_half_with_transaction(
      cx,
      IS_TX_HALF,
      Ordering::Acquire,
      Ordering::AcqRel,
      Inner::take_index_try,
      Inner::take_index_finalize,
    )
  }
}

impl<T, E> Drop for Receiver<T, E> {
  #[inline]
  fn drop(&mut self) {
    self.inner.close_half(IS_TX_HALF);
  }
}

impl<T, E> Inner<T, E> {
  pub(super) fn take_index(&self, state: &mut usize, count: usize) -> usize {
    let begin = *state >> INDEX_BITS & INDEX_MASK;
    *state >>= INDEX_BITS << 1;
    *state <<= INDEX_BITS;
    *state |= begin.wrapping_add(1).wrapping_rem(self.buffer.cap());
    *state <<= INDEX_BITS;
    *state |= count.wrapping_sub(1);
    begin
  }

  pub(super) fn get_count(state: usize) -> usize {
    state & INDEX_MASK
  }

  fn take_index_try(&self, state: &mut usize) -> Option<Result<usize, ()>> {
    let count = Self::get_count(*state);
    if count != 0 {
      Some(Ok(self.take_index(state, count)))
    } else if *state & COMPLETE == 0 {
      None
    } else {
      Some(Err(()))
    }
  }

  fn take_index_finalize(
    &self,
    value: Result<usize, ()>,
  ) -> Option<Result<T, E>> {
    match value {
      Ok(index) => unsafe { Some(Ok(ptr::read(self.buffer.ptr().add(index)))) },
      Err(()) => self.take_err(),
    }
  }
}
