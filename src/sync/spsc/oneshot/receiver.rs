use super::{Inner, COMPLETE, RX_LOCK};
use crate::sync::spsc::SpscInner;
use alloc::sync::Arc;
use core::{
  fmt,
  future::Future,
  pin::Pin,
  sync::atomic::Ordering::*,
  task::{LocalWaker, Poll},
};
use failure::{Backtrace, Fail};

/// The receiving-half of [`oneshot::channel`](super::channel).
#[must_use]
pub struct Receiver<T, E> {
  inner: Arc<Inner<T, E>>,
}

/// Error for `Future` implementation for [`Receiver`](Receiver).
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
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
  type Output = Result<T, RecvError<E>>;

  #[inline]
  fn poll(self: Pin<&mut Self>, lw: &LocalWaker) -> Poll<Self::Output> {
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
  fn recv(&self, lw: &LocalWaker) -> Poll<Result<T, RecvError<E>>> {
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
        unsafe { *self.rx_waker.get() = Some(lw.clone().into_waker()) };
        self.update(state, AcqRel, Relaxed, |state| {
          *state ^= RX_LOCK;
          Ok(*state)
        })
      })
      .and_then(|state| {
        if state & COMPLETE == 0 {
          Ok(Poll::Pending)
        } else {
          Err(())
        }
      })
      .unwrap_or_else(|()| {
        Poll::Ready(unsafe { &mut *self.data.get() }.take().map_or_else(
          || Err(RecvError::Canceled),
          |x| x.map_err(RecvError::Complete),
        ))
      })
  }
}

impl<E> Fail for RecvError<E>
where
  E: fmt::Display + fmt::Debug + Send + Sync + 'static,
{
  fn cause(&self) -> Option<&dyn Fail> {
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
