use super::{Inner, COMPLETE};
use crate::sync::spsc::SpscInner;
use alloc::sync::Arc;
use core::{
    fmt,
    future::Future,
    pin::Pin,
    sync::atomic::Ordering,
    task::{Context, Poll},
};

const IS_TX_HALF: bool = false;

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

    /// Gracefully close this `Receiver`, preventing sending any future
    /// messages.
    #[inline]
    pub fn close(&mut self) {
        self.inner.close_half(IS_TX_HALF)
    }

    /// Attempts to receive a value outside of the context of a task.
    #[inline]
    pub fn try_recv(&mut self) -> Result<Option<T>, RecvError<E>> {
        self.inner.try_recv()
    }
}

impl<T, E> Future for Receiver<T, E> {
    type Output = Result<T, RecvError<E>>;

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.inner.poll_half(
            cx,
            IS_TX_HALF,
            Ordering::Acquire,
            Ordering::AcqRel,
            Inner::take,
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
    fn try_recv(&self) -> Result<Option<T>, RecvError<E>> {
        let state = self.state_load(Ordering::Acquire);
        if state & COMPLETE == 0 {
            Ok(None)
        } else {
            unsafe { &mut *self.data.get() }.take().map_or_else(
                || Err(RecvError::Canceled),
                |value| value.map(Some).map_err(RecvError::Complete),
            )
        }
    }

    fn take(&self, state: u8) -> Poll<Result<T, RecvError<E>>> {
        if state & COMPLETE == 0 {
            Poll::Pending
        } else {
            Poll::Ready(unsafe { &mut *self.data.get() }.take().map_or_else(
                || Err(RecvError::Canceled),
                |value| value.map_err(RecvError::Complete),
            ))
        }
    }
}

impl<E: fmt::Display> fmt::Display for RecvError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecvError::Canceled => write!(f, "Sender is dropped."),
            RecvError::Complete(err) => write!(f, "Received an error: {}", err),
        }
    }
}
