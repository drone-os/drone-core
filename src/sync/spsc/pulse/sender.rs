use super::{Inner, COMPLETE, OPTION_BITS, RX_WAKER_STORED};
use crate::sync::spsc::{SpscInner, SpscInnerErr};
use alloc::sync::Arc;
use core::{
    fmt,
    sync::atomic::Ordering,
    task::{Context, Poll},
};

const IS_TX_HALF: bool = true;

/// The sending-half of [`pulse::channel`](super::channel).
pub struct Sender<E> {
    inner: Arc<Inner<E>>,
}

/// The error type returned from [`Sender::send`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SendError {
    /// The corresponding [`Receiver`](super::Receiver) is dropped.
    Canceled,
    /// The pulse counter overflow.
    Overflow,
}

impl<E> Sender<E> {
    pub(super) fn new(inner: Arc<Inner<E>>) -> Self {
        Self { inner }
    }

    /// Sends the `pulses` number of pulses to the receiving half.
    ///
    /// Returns an error if the receiver was dropped or there is the counter
    /// overflow.
    #[inline]
    pub fn send(&mut self, pulses: usize) -> Result<(), SendError> {
        self.inner.send(pulses)
    }

    /// Completes this channel with an `Err` result.
    ///
    /// This function will consume `self` and indicate to the other end, the
    /// [`Receiver`](super::Receiver), that the channel is closed.
    ///
    /// If the value is successfully enqueued for the remote end to receive,
    /// then `Ok(())` is returned. If the receiving end was dropped before this
    /// function was called, however, then `Err` is returned with the value
    /// provided.
    #[inline]
    pub fn send_err(self, err: E) -> Result<(), E> {
        self.inner.send_err(err)
    }

    /// Polls this `Sender` half to detect whether its associated
    /// [`Receiver`](super::Receiver) with has been dropped.
    ///
    /// # Return values
    ///
    /// If `Ok(Ready)` is returned then the associated `Receiver` has been
    /// dropped.
    ///
    /// If `Ok(Pending)` is returned then the associated `Receiver` is still
    /// alive and may be able to receive pulses if sent. The current task,
    /// however, is scheduled to receive a notification if the corresponding
    /// `Receiver` goes away.
    #[inline]
    pub fn poll_canceled(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        self.inner.poll_half(
            cx,
            IS_TX_HALF,
            Ordering::Relaxed,
            Ordering::Release,
            Inner::take_cancel,
        )
    }

    /// Tests to see whether this `Sender`'s corresponding `Receiver` has been
    /// dropped.
    ///
    /// Unlike [`poll_canceled`](Sender::poll_canceled), this function does not
    /// enqueue a task for wakeup upon cancellation, but merely reports the
    /// current state, which may be subject to concurrent modification.
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
    fn send(&self, pulses: usize) -> Result<(), SendError> {
        let state = self.state_load(Ordering::Acquire);
        self.transaction(state, Ordering::Acquire, Ordering::Acquire, |state| {
            if *state & COMPLETE != 0 {
                return Err(SendError::Canceled);
            }
            let pulses = pulses.checked_shl(OPTION_BITS).ok_or(SendError::Overflow)?;
            *state = state.checked_add(pulses).ok_or(SendError::Overflow)?;
            Ok(*state)
        })
        .map(|state| {
            if state & RX_WAKER_STORED != 0 {
                unsafe { (*self.rx_waker.get()).get_ref().wake_by_ref() };
            }
        })
    }
}

impl fmt::Display for SendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SendError::Canceled => write!(f, "Receiver is dropped."),
            SendError::Overflow => write!(f, "Channel buffer overflow."),
        }
    }
}
