use alloc::sync::Arc;
use core::fmt;
use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::Ordering;
use core::task::{Context, Poll};

use super::{Inner, COMPLETE};
use crate::sync::spsc::SpscInner;

const IS_TX_HALF: bool = false;

/// The receiving-half of [`oneshot::channel`](super::channel).
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Receiver<T> {
    inner: Arc<Inner<T>>,
}

/// Error returned from a [`Receiver`] when the corresponding
/// [`Sender`](super::Sender) is dropped.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Canceled;

impl<T> Receiver<T> {
    pub(super) fn new(inner: Arc<Inner<T>>) -> Self {
        Self { inner }
    }

    /// Gracefully close this receiver, preventing any subsequent attempts to
    /// send to it.
    ///
    /// Any `send` operation which happens after this method returns is
    /// guaranteed to fail. After calling this method, you can use
    /// [`Receiver::poll`](core::future::Future::poll) to determine whether a
    /// message had previously been sent.
    #[inline]
    pub fn close(&mut self) {
        self.inner.close_half(IS_TX_HALF);
    }

    /// Attempts to receive a message outside of the context of a task.
    ///
    /// Does not schedule a task wakeup or have any other side effects.
    ///
    /// A return value of `Ok(None)` must be considered immediately stale (out
    /// of date) unless [`close`](Receiver::close) has been called first.
    ///
    /// Returns an error if the sender was dropped.
    #[inline]
    pub fn try_recv(&mut self) -> Result<Option<T>, Canceled> {
        self.inner.try_recv()
    }
}

impl<T> Future for Receiver<T> {
    type Output = Result<T, Canceled>;

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.inner.poll_half(cx, IS_TX_HALF, Ordering::Acquire, Ordering::AcqRel, Inner::take)
    }
}

impl<T> Drop for Receiver<T> {
    #[inline]
    fn drop(&mut self) {
        self.inner.close_half(IS_TX_HALF);
    }
}

impl<T> Inner<T> {
    fn try_recv(&self) -> Result<Option<T>, Canceled> {
        let state = self.state_load(Ordering::Acquire);
        if state & COMPLETE == 0 {
            Ok(None)
        } else {
            unsafe { &mut *self.data.get() }.take().ok_or(Canceled).map(Some)
        }
    }

    fn take(&self, state: u8) -> Poll<Result<T, Canceled>> {
        if state & COMPLETE == 0 {
            Poll::Pending
        } else {
            Poll::Ready(unsafe { &mut *self.data.get() }.take().ok_or(Canceled))
        }
    }
}

impl fmt::Display for Canceled {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "oneshot canceled")
    }
}
