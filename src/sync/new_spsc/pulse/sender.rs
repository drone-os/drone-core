use core::marker::PhantomData;
use core::pin::Pin;
use core::ptr::NonNull;
use core::task::{Context, Poll};
use core::{fmt, mem};

use futures::prelude::*;

use super::receiver::Receiver;
use super::{
    Shared, CLOSED, ERR_STORED, HALF_DROPPED, PARAM_BITS, RX_WAKER_STORED, TX_WAKER_STORED,
};

/// The sending-half of [`pulse::channel`](super::channel).
pub struct Sender<E> {
    pub(super) ptr: NonNull<Shared<E>>,
    phantom: PhantomData<Shared<E>>,
}

/// A future that resolves when the receiving end of a channel has hung up.
///
/// This is an `.await`-friendly interface around
/// [`poll_canceled`](Sender::poll_canceled).
#[must_use = "futures do nothing unless you `.await` or poll them"]
#[derive(Debug)]
pub struct Cancellation<'a, E> {
    sender: &'a mut Sender<E>,
}

/// The error type returned from [`Sender::send`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SendError {
    /// The pulses could not be sent on the channel because of the pulse counter
    /// overflow.
    Full,
    /// The corresponding [`Receiver`] is dropped.
    Canceled,
}

impl<E> Sender<E> {
    pub(super) fn new(ptr: NonNull<Shared<E>>) -> Self {
        Self { ptr, phantom: PhantomData }
    }

    /// Sends `pulses` number of pulses on this channel.
    ///
    /// If the pulses are successfully enqueued for the remote end to receive,
    /// then `Ok(())` is returned. If the receiving end was dropped before this
    /// function is called, or the internal counter is full, then
    /// `Err(SendError)` is returned.
    pub fn send(&mut self, pulses: usize) -> Result<(), SendError> {
        unsafe {
            let pulses = pulses.checked_shl(PARAM_BITS).ok_or(SendError::Full)?;
            let mut overflow = false;
            let state = load_modify_state!(self.ptr, Acquire, Acquire, |state| {
                state.checked_add(pulses).unwrap_or_else(|| {
                    overflow = true;
                    state
                })
            });
            if overflow {
                return Err(SendError::Full);
            }
            if state & CLOSED != 0 {
                return Err(SendError::Canceled);
            }
            if state & RX_WAKER_STORED != 0 {
                (*self.ptr.as_ref().rx_waker.get()).assume_init_ref().wake_by_ref();
            }
            Ok(())
        }
    }

    /// Completes this channel with an error result.
    ///
    /// This function will consume `self` and indicate to the other end, the
    /// [`Receiver`], that the value provided is the final error of this
    /// channel.
    ///
    /// If the value is successfully enqueued for the remote end to receive,
    /// then `Ok(())` is returned. If the receiving end was dropped before this
    /// function was called, however, then `Err(err)` is returned.
    pub fn send_err(self, err: E) -> Result<(), E> {
        unsafe {
            let Self { ptr, .. } = self;
            mem::forget(self);
            (*ptr.as_ref().err.get()).write(err);
            let state =
                load_modify_state!(ptr, Acquire, AcqRel, |state| state | ERR_STORED | HALF_DROPPED);
            if state & RX_WAKER_STORED != 0 {
                let waker = (*ptr.as_ref().rx_waker.get()).assume_init_read();
                if state & CLOSED == 0 {
                    waker.wake();
                }
            }
            if state & CLOSED != 0 {
                let err = (*ptr.as_ref().err.get()).assume_init_read();
                if state & HALF_DROPPED != 0 {
                    drop(Box::from_raw(ptr.as_ptr()));
                }
                return Err(err);
            }
            Ok(())
        }
    }

    /// Polls this `Sender` half to detect whether its associated [`Receiver`]
    /// has been dropped.
    ///
    /// # Return values
    ///
    /// If `Ready(())` is returned then the associated `Receiver` has been
    /// dropped, which means any work required for sending should be canceled.
    ///
    /// If `Pending` is returned then the associated `Receiver` is still alive
    /// and may be able to receive pulses or an error if sent. The current task,
    /// however, is scheduled to receive a notification if the corresponding
    /// `Receiver` goes away.
    pub fn poll_canceled(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        unsafe {
            let mut state = load_state!(self.ptr, Relaxed);
            if state & CLOSED != 0 {
                return Poll::Ready(());
            }
            if state & TX_WAKER_STORED == 0 {
                (*self.ptr.as_ref().tx_waker.get()).write(cx.waker().clone());
                state = modify_state!(self.ptr, Acquire, AcqRel, |state| state | TX_WAKER_STORED);
                if state & CLOSED != 0 {
                    (*self.ptr.as_ref().tx_waker.get()).assume_init_read();
                    return Poll::Ready(());
                }
            }
            Poll::Pending
        }
    }

    /// Creates a future that resolves when this `Sender`'s corresponding
    /// [`Receiver`] half has hung up.
    ///
    /// This is a utility wrapping [`poll_canceled`](Sender::poll_canceled) to
    /// expose a [`Future`](core::future::Future).
    pub fn cancellation(&mut self) -> Cancellation<'_, E> {
        Cancellation { sender: self }
    }

    /// Tests to see whether this `Sender`'s corresponding `Receiver` has been
    /// dropped.
    ///
    /// Unlike [`poll_canceled`](Sender::poll_canceled), this function does not
    /// enqueue a task for wakeup upon cancellation, but merely reports the
    /// current state, which may be subject to concurrent modification.
    pub fn is_canceled(&self) -> bool {
        unsafe {
            let state = load_state!(self.ptr, Relaxed);
            state & CLOSED != 0
        }
    }

    /// Tests to see whether this `Sender` is connected to the given `Receiver`.
    /// That is, whether they were created by the same call to `channel`.
    pub fn is_connected_to(&self, receiver: &Receiver<E>) -> bool {
        self.ptr.as_ptr() == receiver.ptr.as_ptr()
    }
}

impl<E> Drop for Sender<E> {
    fn drop(&mut self) {
        unsafe {
            let state =
                load_modify_state!(self.ptr, Relaxed, Acquire, |state| state | HALF_DROPPED);
            if state & RX_WAKER_STORED != 0 {
                let waker = (*self.ptr.as_ref().rx_waker.get()).assume_init_read();
                if state & HALF_DROPPED == 0 {
                    waker.wake();
                    return;
                }
            }
            if state & HALF_DROPPED != 0 {
                drop(Box::from_raw(self.ptr.as_ptr()));
            }
        }
    }
}

impl<E> fmt::Debug for Sender<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Sender").finish_non_exhaustive()
    }
}

impl<E> Future for Cancellation<'_, E> {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        self.sender.poll_canceled(cx)
    }
}

impl fmt::Display for SendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SendError::Full => write!(f, "send failed because channel is full"),
            SendError::Canceled => write!(f, "send failed because receiver is gone"),
        }
    }
}
