use super::{Inner, COMPLETE, RX_LOCK};
use alloc::arc::Arc;
use core::sync::atomic::Ordering::*;
use futures::task;
use sync::spsc::SpscInner;

/// The receiving-half of [`oneshot::channel`].
///
/// [`oneshot::channel`]: fn.channel.html
#[must_use]
pub struct Receiver<T, E> {
  inner: Arc<Inner<T, E>>,
}

/// Error returned from [`Receiver::poll`].
///
/// [`Receiver::poll`]: struct.Receiver.html#method.poll
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecvError<E> {
  /// The corresponding [`Sender`] is dropped.
  ///
  /// [`Sender`]: struct.Sender.html
  Canceled,
  /// The corresponding [`Sender`] is completed with an error.
  ///
  /// [`Sender`]: struct.Sender.html
  Complete(E),
}

impl<T, E> Receiver<T, E> {
  #[inline(always)]
  pub(super) fn new(inner: Arc<Inner<T, E>>) -> Self {
    Self { inner }
  }

  /// Gracefully close this [`Receiver`], preventing sending any future
  /// messages.
  ///
  /// [`Receiver`]: struct.Receiver.html
  #[inline(always)]
  pub fn close(&mut self) {
    self.inner.close_rx()
  }
}

impl<T, E> Future for Receiver<T, E> {
  type Item = T;
  type Error = RecvError<E>;

  #[inline(always)]
  fn poll(&mut self) -> Poll<T, RecvError<E>> {
    self.inner.recv()
  }
}

impl<T, E> Drop for Receiver<T, E> {
  #[inline(always)]
  fn drop(&mut self) {
    self.inner.drop_rx();
  }
}

impl<T, E> Inner<T, E> {
  fn recv(&self) -> Poll<T, RecvError<E>> {
    self
      .update(self.state_load(Acquire), Acquire, Acquire, |state| {
        if *state & (COMPLETE | RX_LOCK) != 0 {
          Err(())
        } else {
          *state |= RX_LOCK;
          Ok(*state)
        }
      })
      .and_then(|state| {
        unsafe { *self.rx_task.get() = Some(task::current()) };
        self.update(state, AcqRel, Relaxed, |state| {
          *state ^= RX_LOCK;
          Ok(*state)
        })
      })
      .and_then(|state| {
        if state & COMPLETE == 0 {
          Ok(Async::NotReady)
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
