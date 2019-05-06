use super::{Inner, COMPLETE, LOCK_BITS, LOCK_MASK};
use crate::sync::spsc::{SpscInner, SpscInnerErr};
use alloc::sync::Arc;
use core::{
  num::NonZeroUsize,
  pin::Pin,
  sync::atomic::Ordering,
  task::{Context, Poll},
};
use futures::stream::Stream;

const IS_TX_HALF: bool = false;

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
    self.inner.close_half(IS_TX_HALF)
  }

  /// Polls this [`Receiver`] half for all values at once.
  pub fn poll_all(
    self: Pin<&mut Self>,
    cx: &mut Context<'_>,
  ) -> Poll<Option<Result<NonZeroUsize, E>>> {
    self.inner.poll_half_with_transaction(
      cx,
      IS_TX_HALF,
      Ordering::Acquire,
      Ordering::AcqRel,
      Inner::<E>::take_all_try,
      Inner::take_finalize,
    )
  }
}

impl<E> Stream for Receiver<E> {
  type Item = Result<(), E>;

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
      Inner::<E>::take_try,
      Inner::take_finalize,
    )
  }
}

impl<E> Drop for Receiver<E> {
  #[inline]
  fn drop(&mut self) {
    self.inner.close_half(IS_TX_HALF);
  }
}

impl<E> Inner<E> {
  fn take_try(&self, state: &mut usize) -> Option<Result<(), ()>> {
    let lock = *state & LOCK_MASK;
    *state >>= LOCK_BITS;
    let took = if *state != 0 {
      *state = state.wrapping_sub(1);
      Some(Ok(()))
    } else if lock & COMPLETE == 0 {
      None
    } else {
      Some(Err(()))
    };
    *state <<= LOCK_BITS;
    *state |= lock;
    took
  }

  fn take_all_try(
    &self,
    state: &mut usize,
  ) -> Option<Result<NonZeroUsize, ()>> {
    let value = *state >> LOCK_BITS;
    *state &= LOCK_MASK;
    if let Some(value) = NonZeroUsize::new(value) {
      Some(Ok(value))
    } else if *state & COMPLETE == 0 {
      None
    } else {
      Some(Err(()))
    }
  }

  fn take_finalize<T>(&self, value: Result<T, ()>) -> Option<Result<T, E>> {
    match value {
      Ok(value) => Some(Ok(value)),
      Err(()) => self.take_err(),
    }
  }
}
