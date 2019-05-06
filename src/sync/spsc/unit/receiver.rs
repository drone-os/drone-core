use super::{Inner, COMPLETE, LOCK_BITS, LOCK_MASK, RX_LOCK};
use crate::sync::spsc::SpscInner;
use alloc::sync::Arc;
use core::{
  num::NonZeroUsize,
  pin::Pin,
  sync::atomic::Ordering::*,
  task::{Context, Poll},
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

  /// Polls this [`Receiver`] half for all values at once.
  pub fn poll_all(
    self: Pin<&mut Self>,
    cx: &mut Context<'_>,
  ) -> Poll<Option<Result<NonZeroUsize, E>>> {
    self.inner.recv(cx, Inner::<E>::take_all)
  }
}

impl<E> Stream for Receiver<E> {
  type Item = Result<(), E>;

  #[inline]
  fn poll_next(
    self: Pin<&mut Self>,
    cx: &mut Context<'_>,
  ) -> Poll<Option<Self::Item>> {
    self.inner.recv(cx, Inner::<E>::take)
  }
}

impl<E> Drop for Receiver<E> {
  #[inline]
  fn drop(&mut self) {
    self.inner.drop_rx();
  }
}

impl<E> Inner<E> {
  fn recv<T>(
    &self,
    cx: &mut Context<'_>,
    take: impl Fn(&mut usize) -> Option<T>,
  ) -> Poll<Option<Result<T, E>>> {
    let some_value = |value| Ok(Poll::Ready(Some(Ok(value))));
    self
      .update(self.state_load(Acquire), Acquire, Acquire, |state| {
        if let Some(value) = take(state) {
          Ok(Ok(value))
        } else if *state & COMPLETE == 0 {
          *state |= RX_LOCK;
          Ok(Err(*state))
        } else {
          Err(())
        }
      })
      .and_then(|state| {
        let no_value = |state| {
          unsafe {
            (*self.rx_waker.get()).get_or_insert_with(|| cx.waker().clone());
          }
          self
            .update(state, AcqRel, Relaxed, |state| {
              *state ^= RX_LOCK;
              if let Some(value) = take(state) {
                Ok(Ok(value))
              } else {
                Ok(Err(*state))
              }
            })
            .and_then(|state| {
              let no_value = |state| {
                if state & COMPLETE == 0 {
                  Ok(Poll::Pending)
                } else {
                  Err(())
                }
              };
              state.map_or_else(no_value, some_value)
            })
        };
        state.map_or_else(no_value, some_value)
      })
      .unwrap_or_else(|()| {
        Poll::Ready(unsafe { &mut *self.err.get() }.take().map(Err))
      })
  }

  #[inline]
  fn take(state: &mut usize) -> Option<()> {
    let lock = *state & LOCK_MASK;
    *state >>= LOCK_BITS;
    let took = if *state == 0 {
      None
    } else {
      *state = state.wrapping_sub(1);
      Some(())
    };
    *state <<= LOCK_BITS;
    *state |= lock;
    took
  }

  #[inline]
  fn take_all(state: &mut usize) -> Option<NonZeroUsize> {
    let value = *state >> LOCK_BITS;
    *state &= LOCK_MASK;
    NonZeroUsize::new(value)
  }
}
