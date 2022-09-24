use core::fmt;
use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};

use futures::prelude::*;

/// The receiving-half of [`oneshot::channel`](super::channel).
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Receiver<T> {
    _marker: PhantomData<T>,
}

/// Error returned from a [`Receiver`] when the corresponding
/// [`Sender`](super::Sender) is dropped.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Canceled;

impl<T> Receiver<T> {
    /// Gracefully close this receiver, preventing any subsequent attempts to
    /// send to it.
    ///
    /// Any `send` operation which happens after this method returns is
    /// guaranteed to fail. After calling this method, you can use
    /// [`Receiver::poll`](core::future::Future::poll) to determine whether a
    /// message had previously been sent.
    #[inline]
    pub fn close(&mut self) {
        todo!()
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
        todo!()
    }
}

impl<T> Future for Receiver<T> {
    type Output = Result<T, Canceled>;

    #[inline]
    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        todo!()
    }
}

impl<T> Drop for Receiver<T> {
    #[inline]
    fn drop(&mut self) {
        todo!()
    }
}

impl fmt::Display for Canceled {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "oneshot canceled")
    }
}
