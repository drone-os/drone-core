use super::{Inner, COMPLETE, LOCK_BITS, LOCK_MASK, RX_WAKER_STORED};
use crate::sync::spsc::{SpscInner, SpscInnerErr};
use alloc::sync::Arc;
use core::{
    pin::Pin,
    sync::atomic::Ordering,
    task::{Context, Poll},
};
use failure::Fail;

const IS_TX_HALF: bool = true;

/// The sending-half of [`unit::channel`](super::channel).
pub struct Sender<E> {
    inner: Arc<Inner<E>>,
}

/// Error returned from [`Sender::send`](Sender::send).
#[derive(Debug, Fail)]
pub enum SendError {
    /// The corresponding [`Receiver`](super::Receiver) is dropped.
    #[fail(display = "Receiver is dropped.")]
    Canceled,
    /// Counter overflow.
    #[fail(display = "Channel buffer overflow.")]
    Overflow,
}

impl<E> Sender<E> {
    #[inline]
    pub(super) fn new(inner: Arc<Inner<E>>) -> Self {
        Self { inner }
    }

    /// Sends a unit across the channel.
    #[inline]
    pub fn send(&mut self) -> Result<(), SendError> {
        self.inner.send()
    }

    /// Completes this stream with an error.
    ///
    /// If the value is successfully enqueued, then `Ok(())` is returned. If the
    /// receiving end was dropped before this function was called, then `Err` is
    /// returned with the value provided.
    #[inline]
    pub fn send_err(self, err: E) -> Result<(), E> {
        self.inner.send_err(err)
    }

    /// Polls this [`Sender`] half to detect whether the [`Receiver`] this has
    /// paired with has gone away.
    ///
    /// # Panics
    ///
    /// Like `Future::poll`, this function will panic if it's not called from
    /// within the context of a task. In other words, this should only ever be
    /// called from inside another future.
    ///
    /// If you're calling this function from a context that does not have a
    /// task, then you can use the [`is_canceled`] API instead.
    ///
    /// [`Sender`]: Sender
    /// [`Receiver`]: super::Receiver
    /// [`is_canceled`]: Sender::is_canceled
    #[inline]
    pub fn poll_cancel(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        self.inner.poll_half(
            cx,
            IS_TX_HALF,
            Ordering::Relaxed,
            Ordering::Release,
            Inner::take_cancel,
        )
    }

    /// Tests to see whether this [`Sender`]'s corresponding [`Receiver`] has
    /// gone away.
    ///
    /// [`Sender`]: Sender
    /// [`Receiver`]: super::Receiver
    #[inline]
    pub fn is_canceled(&self) -> bool {
        self.inner.is_canceled(Ordering::Relaxed)
    }
}

impl<E> Drop for Sender<E> {
    #[inline]
    fn drop(&mut self) {
        self.inner.close_half(IS_TX_HALF);
    }
}

impl<E> Inner<E> {
    fn send(&self) -> Result<(), SendError> {
        let state = self.state_load(Ordering::Acquire);
        self.transaction(state, Ordering::Acquire, Ordering::Acquire, |state| {
            if *state & COMPLETE != 0 {
                return Err(SendError::Canceled);
            }
            let lock = *state & LOCK_MASK;
            *state = (*state as isize >> LOCK_BITS) as usize;
            *state = state.wrapping_add(1);
            if *state == 0 {
                return Err(SendError::Overflow);
            }
            *state <<= LOCK_BITS;
            *state |= lock;
            Ok(*state)
        })
        .map(|state| {
            if state & RX_WAKER_STORED != 0 {
                unsafe { (*self.rx_waker.get()).get_ref().wake_by_ref() };
            }
        })
    }
}
