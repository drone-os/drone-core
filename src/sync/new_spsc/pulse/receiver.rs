use core::fmt;
use core::marker::PhantomData;
use core::num::NonZeroUsize;
use core::pin::Pin;
use core::ptr::NonNull;
use core::task::{Context, Poll};

use futures::prelude::*;
use futures::stream::FusedStream;

use super::{
    Shared, CLOSED, CLOSED_WITH_ERR, CLOSED_WITH_ERR_SHIFT, ERR_STORED, ERR_STORED_SHIFT,
    HALF_DROPPED, PARAM_BITS, RX_WAKER_STORED, TX_WAKER_STORED,
};

/// The receiving-half of [`pulse::channel`](super::channel).
pub struct Receiver<E> {
    pub(super) ptr: NonNull<Shared<E>>,
    phantom: PhantomData<Shared<E>>,
}

/// Error returned from a [`Receiver`] when the corresponding
/// [`Sender`](super::Sender) is dropped.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Canceled;

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
            let state = load_modify_state!(self.ptr, Relaxed, Acquire, |state| state
                | CLOSED
                | (state >> ERR_STORED_SHIFT & 1) << CLOSED_WITH_ERR_SHIFT);
            if state & CLOSED == 0 && state & TX_WAKER_STORED != 0 {
                let waker = (*self.ptr.as_ref().tx_waker.get()).assume_init_read();
                if state & HALF_DROPPED == 0 {
                    waker.wake();
                }
            }
        }
    }

    /// Attempts to receive pulses of an error message outside of the context of
    /// a task.
    ///
    /// Does not schedule a task wakeup or have any other side effects.
    ///
    /// A return value of `Ok(None)` must be considered immediately stale (out
    /// of date) unless [`close`](Receiver::close) has been called first.
    ///
    /// Returns an error if the sender was dropped.
    pub fn try_next(&mut self) -> Result<Option<Result<NonZeroUsize, E>>, Canceled> {
        unsafe {
            let mut state = load_modify_state!(self.ptr, Relaxed, Acquire, |state| state
                & (1 << PARAM_BITS) - 1
                & !ERR_STORED);
            let err_available =
                state & ERR_STORED != 0 && (state & CLOSED == 0 || state & CLOSED_WITH_ERR != 0);
            if let Some(value) = NonZeroUsize::new(state >> PARAM_BITS) {
                if err_available {
                    modify_state!(self.ptr, Relaxed, Relaxed, |state| state | ERR_STORED);
                }
                return Ok(Some(Ok(value)));
            }
            if err_available {
                return Ok(Some(Err((*self.ptr.as_ref().err.get()).assume_init_read())));
            }
            if state & HALF_DROPPED != 0 || state & CLOSED != 0 {
                return Err(Canceled);
            }
            Ok(None)
        }
    }
}

impl<E> Stream for Receiver<E> {
    type Item = Result<NonZeroUsize, E>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        unsafe {
            let mut state = load_modify_state!(self.ptr, Relaxed, Acquire, |state| state
                & (1 << PARAM_BITS) - 1
                & !ERR_STORED);
            let err_available =
                state & ERR_STORED != 0 && (state & CLOSED == 0 || state & CLOSED_WITH_ERR != 0);
            if let Some(value) = NonZeroUsize::new(state >> PARAM_BITS) {
                if err_available {
                    modify_state!(self.ptr, Relaxed, Relaxed, |state| state | ERR_STORED);
                }
                return Poll::Ready(Some(Ok(value)));
            }
            if err_available {
                return Poll::Ready(Some(Err((*self.ptr.as_ref().err.get()).assume_init_read())));
            }
            if state & HALF_DROPPED != 0 || state & CLOSED != 0 {
                return Poll::Ready(None);
            }
            if state & RX_WAKER_STORED == 0 {
                (*self.ptr.as_ref().rx_waker.get()).write(cx.waker().clone());
                state = modify_state!(self.ptr, Acquire, AcqRel, |state| state
                    & (1 << PARAM_BITS) - 1
                    & !ERR_STORED
                    | RX_WAKER_STORED);
                if state & HALF_DROPPED != 0 {
                    (*self.ptr.as_ref().rx_waker.get()).assume_init_read();
                }
                if let Some(value) = NonZeroUsize::new(state >> PARAM_BITS) {
                    if state & ERR_STORED != 0 {
                        modify_state!(self.ptr, Relaxed, Relaxed, |state| state | ERR_STORED);
                    }
                    return Poll::Ready(Some(Ok(value)));
                }
                if state & HALF_DROPPED != 0 {
                    if state & ERR_STORED != 0 {
                        return Poll::Ready(Some(Err(
                            (*self.ptr.as_ref().err.get()).assume_init_read()
                        )));
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
            let state = load_state!(self.ptr, Relaxed);
            (state >> PARAM_BITS == 0)
                && (state & HALF_DROPPED != 0 || state & CLOSED != 0)
                && (state & ERR_STORED == 0 || state & CLOSED != 0 && state & CLOSED_WITH_ERR == 0)
        }
    }
}

impl<E> Drop for Receiver<E> {
    fn drop(&mut self) {
        unsafe {
            let state = load_modify_state!(self.ptr, Relaxed, Acquire, |state| state
                | CLOSED
                | HALF_DROPPED);
            if state & ERR_STORED != 0 && (state & CLOSED == 0 || state & CLOSED_WITH_ERR != 0) {
                (*self.ptr.as_ref().err.get()).assume_init_read();
            }
            if state & CLOSED == 0 && state & TX_WAKER_STORED != 0 {
                let waker = (*self.ptr.as_ref().tx_waker.get()).assume_init_read();
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

impl<E: fmt::Debug> fmt::Debug for Receiver<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Receiver").finish_non_exhaustive()
    }
}

impl fmt::Display for Canceled {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "pulse canceled")
    }
}
