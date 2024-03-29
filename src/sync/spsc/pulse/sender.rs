use super::receiver::Receiver;
use super::{
    Shared, State, CAPACITY, CLOSED, ERR_STORED, HALF_DROPPED, PARAM_BITS, RX_WAKER_STORED,
    TX_WAKER_STORED,
};
use core::cell::UnsafeCell;
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::pin::Pin;
use core::ptr::NonNull;
use core::task::{Context, Poll, Waker};
use core::{fmt, mem};
use futures::prelude::*;

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
    /// The corresponding [`Receiver`] is closed or dropped.
    Canceled,
}

impl<E> Sender<E> {
    pub(super) fn new(ptr: NonNull<Shared<E>>) -> Self {
        Self { ptr, phantom: PhantomData }
    }

    /// Sends `pulses` number of pulses on this channel.
    ///
    /// If the pulses are successfully enqueued for the remote end to receive,
    /// then `Ok(())` is returned. If the receiving end is closed, then
    /// `Err(SendError::Canceled)` is returned. If the internal counter doesn't
    /// have enough space to add `pulses` without overflow, then
    /// `Err(SendError::Full)` is returned.
    pub fn send(&mut self, mut pulses: usize) -> Result<(), SendError> {
        unsafe {
            if pulses > CAPACITY - 1 {
                return Err(SendError::Full);
            }
            pulses <<= PARAM_BITS;
            let state = load_modify_atomic!(self.state(), Acquire, Acquire, |state| state
                .checked_add(pulses)
                .unwrap_or(state));
            if state.checked_add(pulses).is_none() {
                return Err(SendError::Full);
            }
            if state & CLOSED != 0 {
                return Err(SendError::Canceled);
            }
            if state & RX_WAKER_STORED != 0 {
                (*self.rx_waker().get()).assume_init_ref().wake_by_ref();
            }
            Ok(())
        }
    }

    /// Sends `pulses` number of pulses on this channel, possibly saturating the
    /// internal counter instead of returning an error on overflow.
    ///
    /// If the pulses are successfully enqueued for the remote end to receive,
    /// then `Ok(())` is returned. If the receiving end is closed, then
    /// `Err(SendError::Canceled)` is returned.
    pub fn saturating_send(&mut self, mut pulses: usize) -> Result<(), SendError> {
        unsafe {
            if pulses > CAPACITY - 1 {
                pulses = (CAPACITY - 1) << PARAM_BITS;
            } else {
                pulses <<= PARAM_BITS;
            }
            let state = load_modify_atomic!(self.state(), Acquire, Acquire, |state| state
                .checked_add(pulses)
                .unwrap_or(state | (CAPACITY - 1) << PARAM_BITS));
            if state & CLOSED != 0 {
                return Err(SendError::Canceled);
            }
            if state & RX_WAKER_STORED != 0 {
                (*self.rx_waker().get()).assume_init_ref().wake_by_ref();
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
            let state = load_modify_atomic!(ptr.as_ref().state, Acquire, AcqRel, |state| state
                | if state & CLOSED == 0 { ERR_STORED } else { 0 }
                | HALF_DROPPED);
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
            let mut state = load_atomic!(self.state(), Relaxed);
            if state & CLOSED != 0 {
                return Poll::Ready(());
            }
            if state & TX_WAKER_STORED == 0 {
                (*self.tx_waker().get()).write(cx.waker().clone());
                state =
                    modify_atomic!(self.state(), Relaxed, Release, |state| state | TX_WAKER_STORED);
                if state & CLOSED != 0 {
                    (*self.tx_waker().get()).assume_init_read();
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
    #[inline]
    pub fn cancellation(&mut self) -> Cancellation<'_, E> {
        Cancellation { sender: self }
    }

    /// Tests to see whether this `Sender`'s corresponding `Receiver` has been
    /// dropped.
    ///
    /// Unlike [`poll_canceled`](Sender::poll_canceled), this function does not
    /// enqueue a task for wakeup upon cancellation, but merely reports the
    /// current state, which may be subject to concurrent modification.
    #[inline]
    pub fn is_canceled(&self) -> bool {
        unsafe {
            let state = load_atomic!(self.state(), Relaxed);
            state & CLOSED != 0
        }
    }

    /// Tests to see whether this `Sender` is connected to the given `Receiver`.
    /// That is, whether they were created by the same call to `channel`.
    #[inline]
    pub fn is_connected_to(&self, receiver: &Receiver<E>) -> bool {
        self.ptr.as_ptr() == receiver.ptr.as_ptr()
    }

    unsafe fn state(&self) -> &State {
        unsafe { &self.ptr.as_ref().state }
    }

    unsafe fn tx_waker(&self) -> &UnsafeCell<MaybeUninit<Waker>> {
        unsafe { &self.ptr.as_ref().tx_waker }
    }

    unsafe fn rx_waker(&self) -> &UnsafeCell<MaybeUninit<Waker>> {
        unsafe { &self.ptr.as_ref().rx_waker }
    }
}

impl<E> Drop for Sender<E> {
    fn drop(&mut self) {
        unsafe {
            let state =
                load_modify_atomic!(self.state(), Relaxed, Acquire, |state| state | HALF_DROPPED);
            if state & RX_WAKER_STORED != 0 {
                let waker = (*self.rx_waker().get()).assume_init_read();
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

    #[inline]
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        self.sender.poll_canceled(cx)
    }
}

impl fmt::Display for SendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Full => write!(f, "send failed because channel is full"),
            Self::Canceled => write!(f, "send failed because receiver is gone"),
        }
    }
}
