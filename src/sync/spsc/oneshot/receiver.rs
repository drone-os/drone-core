use super::{Inner, COMPLETE, RX_LOCK};
use alloc::sync::Arc;
use core::{fmt, sync::atomic::Ordering::*};
use failure::{Backtrace, Fail};
use futures::prelude::*;
use sync::spsc::SpscInner;

/// The receiving-half of [`oneshot::channel`](super::channel).
#[must_use]
pub struct Receiver<T, E> {
  inner: Arc<Inner<T, E>>,
}

/// Error for `Future` implementation for [`Receiver`](Receiver).
#[derive(Debug)]
pub enum RecvError<E> {
  /// The corresponding [`Sender`](super::Sender) is dropped.
  Canceled,
  /// The corresponding [`Sender`](super::Sender) completed with an error.
  Complete(E),
}

impl<T, E> Receiver<T, E> {
  #[inline]
  pub(super) fn new(inner: Arc<Inner<T, E>>) -> Self {
    Self { inner }
  }

  /// Gracefully close this `Receiver`, preventing sending any future messages.
  #[inline]
  pub fn close(&mut self) {
    self.inner.close_rx()
  }
}

impl<T, E> Future for Receiver<T, E> {
  type Item = T;
  type Error = RecvError<E>;

  #[inline]
  fn poll(&mut self, cx: &mut task::Context) -> Poll<T, RecvError<E>> {
    self.inner.recv(cx)
  }
}

impl<T, E> Drop for Receiver<T, E> {
  #[inline]
  fn drop(&mut self) {
    self.inner.drop_rx();
  }
}

impl<T, E> Inner<T, E> {
  fn recv(&self, cx: &mut task::Context) -> Poll<T, RecvError<E>> {
    self
      .update(self.state_load(Acquire), Acquire, Acquire, |state| {
        if *state & (COMPLETE | RX_LOCK) == 0 {
          *state |= RX_LOCK;
          Ok(*state)
        } else {
          Err(())
        }
      })
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

impl<E> Fail for RecvError<E>
where
  E: fmt::Display + fmt::Debug + Send + Sync + 'static,
{
  fn cause(&self) -> Option<&Fail> {
    None
  }

  fn backtrace(&self) -> Option<&Backtrace> {
    None
  }
}

impl<E: fmt::Display> fmt::Display for RecvError<E> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      RecvError::Canceled => write!(f, "Sender is dropped."),
      RecvError::Complete(err) => write!(f, "Received an error: {}", err),
    }
  }
}
