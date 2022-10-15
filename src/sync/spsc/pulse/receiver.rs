use super::{
    Shared, State, CLOSED, ERR_STORED, HALF_DROPPED, PARAM_BITS, RX_WAKER_STORED, TX_WAKER_STORED,
};
use core::cell::UnsafeCell;
use core::fmt;
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::num::NonZeroUsize;
use core::pin::Pin;
use core::ptr::NonNull;
use core::task::{Context, Poll, Waker};
use futures::prelude::*;
use futures::stream::FusedStream;

/// The receiving-half of [`pulse::channel`](super::channel).
pub struct Receiver<E> {
    pub(super) ptr: NonNull<Shared<E>>,
    phantom: PhantomData<Shared<E>>,
}

/// This enumeration is the list of the possible reasons that
/// [`Receiver::try_next`] could not return data when called.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TryNextError {
    /// This channel is currently empty, but the [`Sender`](super::Sender) have
    /// not yet disconnected, so data may yet become available.
    Empty,
    /// The channel’s sending half has become disconnected, and there will never
    /// be any more data received on it.
    Canceled,
}

impl<E> Receiver<E> {
    pub(super) fn new(ptr: NonNull<Shared<E>>) -> Self {
        Self { ptr, phantom: PhantomData }
    }

    /// Closes the receiving half of a channel, without dropping it.
    ///
    /// This prevents any further pulses or error messages from being sent on
    /// the channel while still enabling the receiver to drain pulses or an
    /// error message that is buffered.
    pub fn close(&mut self) {
        unsafe {
            let state = load_modify_atomic!(self.state(), Relaxed, Acquire, |state| state | CLOSED);
            if state & CLOSED == 0 && state & TX_WAKER_STORED != 0 {
                let waker = (*self.tx_waker().get()).assume_init_read();
                if state & HALF_DROPPED == 0 {
                    waker.wake();
                }
            }
        }
    }

    /// Attempts to receive pulses or an error message outside of the context of
    /// a task.
    ///
    /// Does not schedule a task wakeup or have any other side effects.
    ///
    /// A return value of `Err(TryNextError::Empty)` must be considered
    /// immediately stale (out of date) unless [`close`](Receiver::close)
    /// has been called first.
    ///
    /// Returns an error if the counter is full or the sender was dropped.
    pub fn try_next(&mut self) -> Result<Result<NonZeroUsize, E>, TryNextError> {
        unsafe {
            #[cfg_attr(not(feature = "atomics"), allow(unused_mut))]
            let mut state = load_modify_atomic!(self.state(), Relaxed, Acquire, |state| state
                & (1 << PARAM_BITS) - 1
                & !ERR_STORED);
            if let Some(value) = NonZeroUsize::new(state >> PARAM_BITS) {
                if state & ERR_STORED != 0 {
                    modify_atomic!(self.state(), Relaxed, Relaxed, |state| state | ERR_STORED);
                }
                return Ok(Ok(value));
            }
            if state & ERR_STORED != 0 {
                return Ok(Err((*self.err().get()).assume_init_read()));
            }
            if state & HALF_DROPPED != 0 || state & CLOSED != 0 {
                return Err(TryNextError::Canceled);
            }
            Err(TryNextError::Empty)
        }
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

    unsafe fn err(&self) -> &UnsafeCell<MaybeUninit<E>> {
        unsafe { &self.ptr.as_ref().err }
    }
}

impl<E> Stream for Receiver<E> {
    type Item = Result<NonZeroUsize, E>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        unsafe {
            let mut state = load_modify_atomic!(self.state(), Relaxed, Acquire, |state| state
                & (1 << PARAM_BITS) - 1
                & !ERR_STORED);
            if let Some(value) = NonZeroUsize::new(state >> PARAM_BITS) {
                if state & ERR_STORED != 0 {
                    modify_atomic!(self.state(), Relaxed, Relaxed, |state| state | ERR_STORED);
                }
                return Poll::Ready(Some(Ok(value)));
            }
            if state & ERR_STORED != 0 {
                return Poll::Ready(Some(Err((*self.err().get()).assume_init_read())));
            }
            if state & HALF_DROPPED != 0 || state & CLOSED != 0 {
                return Poll::Ready(None);
            }
            if state & RX_WAKER_STORED == 0 {
                (*self.rx_waker().get()).write(cx.waker().clone());
                state = modify_atomic!(self.state(), Acquire, AcqRel, |state| state
                    & (1 << PARAM_BITS) - 1
                    & !ERR_STORED
                    | RX_WAKER_STORED);
                if state & HALF_DROPPED != 0 {
                    (*self.rx_waker().get()).assume_init_read();
                }
                if let Some(value) = NonZeroUsize::new(state >> PARAM_BITS) {
                    if state & ERR_STORED != 0 {
                        modify_atomic!(self.state(), Relaxed, Relaxed, |state| state | ERR_STORED);
                    }
                    return Poll::Ready(Some(Ok(value)));
                }
                if state & HALF_DROPPED != 0 {
                    if state & ERR_STORED != 0 {
                        return Poll::Ready(Some(Err((*self.err().get()).assume_init_read())));
                    }
                    return Poll::Ready(None);
                }
            }
            Poll::Pending
        }
    }
}

impl<E> FusedStream for Receiver<E> {
    fn is_terminated(&self) -> bool {
        unsafe {
            let state = load_atomic!(self.state(), Relaxed);
            (state & HALF_DROPPED != 0 || state & CLOSED != 0)
                && state & ERR_STORED == 0
                && (state >> PARAM_BITS == 0)
        }
    }
}

impl<E> Drop for Receiver<E> {
    fn drop(&mut self) {
        unsafe {
            let state = load_modify_atomic!(self.state(), Relaxed, Acquire, |state| state
                | CLOSED
                | HALF_DROPPED);
            if state & ERR_STORED != 0 {
                (*self.err().get()).assume_init_read();
            }
            if state & CLOSED == 0 && state & TX_WAKER_STORED != 0 {
                let waker = (*self.tx_waker().get()).assume_init_read();
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

impl<E> fmt::Debug for Receiver<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Receiver").finish_non_exhaustive()
    }
}

impl fmt::Display for TryNextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "pulse channel is empty"),
            Self::Canceled => write!(f, "pulse channel is canceled"),
        }
    }
}
