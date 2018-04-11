use super::{Inner, COMPLETE, RX_LOCK};
use alloc::arc::Arc;
use core::sync::atomic::Ordering::*;
use futures::prelude::*;
use sync::spsc::SpscInner;

/// The receiving-half of [`oneshot::channel`](channel).
#[must_use]
pub struct Receiver<T, E> {
  inner: Arc<Inner<T, E>>,
}

/// Error for `Future` implementation for [`Receiver`](Receiver).
#[derive(Debug, Fail)]
pub enum RecvError<E> {
  /// The corresponding [`Sender`](Sender) is dropped.
  #[fail(display = "Sender is dropped.")]
  Canceled,
  /// The corresponding [`Sender`](Sender) completed with an error.
  #[fail(display = "Received an error: {}", _0)]
  Complete(E),
}

impl<T, E> Receiver<T, E> {
  #[inline(always)]
  pub(super) fn new(inner: Arc<Inner<T, E>>) -> Self {
    Self { inner }
  }

  /// Gracefully close this `Receiver`, preventing sending any future messages.
  #[inline(always)]
  pub fn close(&mut self) {
    self.inner.close_rx()
  }
}

impl<T, E> Future for Receiver<T, E> {
  type Item = T;
  type Error = RecvError<E>;

  #[inline(always)]
  fn poll(&mut self, cx: &mut task::Context) -> Poll<T, RecvError<E>> {
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
  fn recv(&self, cx: &mut task::Context) -> Poll<T, RecvError<E>> {
    self
      .update(
        self.state_load(Acquire),
        Acquire,
        Acquire,
        |state| {
          if *state & (COMPLETE | RX_LOCK) != 0 {
            Err(())
          } else {
            *state |= RX_LOCK;
            Ok(*state)
          }
        },
      )
      .and_then(|state| {
        unsafe { *self.rx_waker.get() = Some(cx.waker().clone()) };
        self.update(state, AcqRel, Relaxed, |state| {
          *state ^= RX_LOCK;
          Ok(*state)
        })
      })
      .and_then(|state| {
        if state & COMPLETE == 0 {
          Ok(Async::Pending)
        } else {
          Err(())
        }
      })
      .or_else(|()| {
        let data = unsafe { &mut *self.data.get() };
        data
          .take()
          .ok_or(RecvError::Canceled)
          .and_then(|x| x.map(Async::Ready).map_err(RecvError::Complete))
      })
  }
}
