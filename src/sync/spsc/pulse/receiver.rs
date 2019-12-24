use super::{Inner, COMPLETE, OPTION_BITS};
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

/// The receiving-half of [`pulse::channel`](super::channel).
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Receiver<E> {
    inner: Arc<Inner<E>>,
}

impl<E> Receiver<E> {
    pub(super) fn new(inner: Arc<Inner<E>>) -> Self {
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
        self.inner.close_half(IS_TX_HALF)
    }

    /// Attempts to receive pulses outside of the context of a task.
    ///
    /// Does not schedule a task wakeup or have any other side effects.
    ///
    /// A return value of `Ok(None)` must be considered immediately stale (out
    /// of date) unless [`close`](Receiver::close) has been called first.
    #[inline]
    pub fn try_recv(&mut self) -> Result<Option<NonZeroUsize>, E> {
        self.inner.try_recv(Inner::<E>::take_try)
    }
}

impl<E> Stream for Receiver<E> {
    type Item = Result<NonZeroUsize, E>;

    #[inline]
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
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

#[allow(clippy::unused_self)]
impl<E> Inner<E> {
    fn try_recv<T>(
        &self,
        take_try: fn(&Self, &mut usize) -> Option<Result<T, ()>>,
    ) -> Result<Option<T>, E> {
        let state = self.state_load(Ordering::Acquire);
        self.transaction(state, Ordering::AcqRel, Ordering::Acquire, |state| {
            match take_try(self, state) {
                Some(value) => value.map(Some).map_err(Ok),
                None => Err(Err(())),
            }
        })
        .or_else(|value| value.map_or_else(|()| Ok(None), |()| self.take_err().transpose()))
    }

    fn take_try(&self, state: &mut usize) -> Option<Result<NonZeroUsize, ()>> {
        let value = *state >> OPTION_BITS;
        *state ^= value << OPTION_BITS;
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
