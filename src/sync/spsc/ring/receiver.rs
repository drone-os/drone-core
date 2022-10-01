use core::cell::UnsafeCell;
use core::fmt;
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::pin::Pin;
use core::ptr::NonNull;
use core::task::{Context, Poll, Waker};

use futures::prelude::*;
use futures::stream::FusedStream;

use super::{
    add_cursor, claim_next_unless_empty, get_cursor, get_length, has_flush_waker, has_ready_waker,
    has_waker, Shared, State, CLOSED, COUNT_BITS, ERR_STORED, HALF_DROPPED, RX_WAKER_STORED,
    TX_FLUSH_WAKER_STORED, TX_READY_WAKER_STORED,
};

/// The receiving-half of [`ring::channel`](super::channel).
pub struct Receiver<T, E> {
    pub(super) ptr: NonNull<Shared<T, E>>,
    phantom: PhantomData<Shared<T, E>>,
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

impl<T, E> Receiver<T, E> {
    pub(super) fn new(ptr: NonNull<Shared<T, E>>) -> Self {
        Self { ptr, phantom: PhantomData }
    }

    /// Closes the receiving half of a channel, without dropping it.
    ///
    /// This prevents any further messages from being sent on the channel while
    /// still enabling the receiver to drain messages that are buffered.
    pub fn close(&mut self) {
        unsafe {
            let state = load_modify_atomic!(self.state(), Relaxed, Acquire, |state| state | CLOSED);
            if state & CLOSED == 0 && has_waker(state) {
                let waker = (*self.tx_waker().get()).assume_init_read();
                if state & HALF_DROPPED == 0 {
                    waker.wake();
                }
            }
        }
    }

    /// Attempts to receive a message outside of the context of a task.
    ///
    /// Does not schedule a task wakeup or have any other side effects.
    ///
    /// A return value of `Err(TryNextError::Empty)` must be considered
    /// immediately stale (out of date) unless [`close`](Receiver::close)
    /// has been called first.
    ///
    /// Returns an error if the ring buffer is full or the sender was dropped.
    pub fn try_next(&mut self) -> Result<Result<T, E>, TryNextError> {
        unsafe {
            let state = load_modify_atomic!(self.state(), Relaxed, Acquire, |state| {
                claim_next_unless_empty(state, self.buf().len()) & !ERR_STORED
            });
            let length = get_length(state);
            if length > 0 {
                return Ok(Ok(self.take_value(state, length)));
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

    fn take_value(&self, mut state: usize, length: usize) -> T {
        unsafe {
            let index = get_cursor(state);
            let should_wake = if has_ready_waker(state) {
                length == self.buf().len()
            } else if has_flush_waker(state) {
                length == 1
            } else {
                false
            };
            let mut set_flags = 0;
            if should_wake {
                set_flags |= TX_READY_WAKER_STORED | TX_FLUSH_WAKER_STORED;
            };
            if state & ERR_STORED != 0 {
                set_flags |= ERR_STORED;
            };
            if set_flags != 0 {
                state = modify_atomic!(self.state(), Relaxed, Acquire, |state| state | set_flags);
            }
            if should_wake && state & HALF_DROPPED == 0 {
                (*self.tx_waker().get()).assume_init_ref().wake_by_ref();
            }
            (*self.buf().get_unchecked(index).get()).assume_init_read()
        }
    }

    unsafe fn state(&self) -> &State {
        unsafe { &self.ptr.as_ref().hdr.state }
    }

    unsafe fn tx_waker(&self) -> &UnsafeCell<MaybeUninit<Waker>> {
        unsafe { &self.ptr.as_ref().hdr.tx_waker }
    }

    unsafe fn rx_waker(&self) -> &UnsafeCell<MaybeUninit<Waker>> {
        unsafe { &self.ptr.as_ref().hdr.rx_waker }
    }

    unsafe fn err(&self) -> &UnsafeCell<MaybeUninit<E>> {
        unsafe { &self.ptr.as_ref().hdr.err }
    }

    unsafe fn buf(&self) -> &[UnsafeCell<MaybeUninit<T>>] {
        unsafe { &self.ptr.as_ref().buf }
    }
}

impl<T, E> Stream for Receiver<T, E> {
    type Item = Result<T, E>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        unsafe {
            let mut state = load_modify_atomic!(self.state(), Relaxed, Acquire, |state| {
                claim_next_unless_empty(state, self.buf().len()) & !ERR_STORED
            });
            let length = get_length(state);
            if length > 0 {
                return Poll::Ready(Some(Ok(self.take_value(state, length))));
            }
            if state & ERR_STORED != 0 {
                return Poll::Ready(Some(Err((*self.err().get()).assume_init_read())));
            }
            if state & HALF_DROPPED != 0 || state & CLOSED != 0 {
                return Poll::Ready(None);
            }
            if state & RX_WAKER_STORED == 0 {
                (*self.rx_waker().get()).write(cx.waker().clone());
                state = modify_atomic!(self.state(), Acquire, AcqRel, |state| {
                    claim_next_unless_empty(state, self.buf().len()) & !ERR_STORED | RX_WAKER_STORED
                });
                if state & HALF_DROPPED != 0 {
                    (*self.rx_waker().get()).assume_init_read();
                }
                let length = get_length(state);
                if length > 0 {
                    return Poll::Ready(Some(Ok(self.take_value(state, length))));
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

impl<T, E> FusedStream for Receiver<T, E> {
    fn is_terminated(&self) -> bool {
        unsafe {
            let state = load_atomic!(self.state(), Relaxed);
            (state & HALF_DROPPED != 0 || state & CLOSED != 0)
                && state & ERR_STORED == 0
                && get_length(state) == 0
        }
    }
}

impl<T, E> Drop for Receiver<T, E> {
    fn drop(&mut self) {
        unsafe {
            let state = load_modify_atomic!(self.state(), Relaxed, Acquire, |state| {
                state << COUNT_BITS >> COUNT_BITS | CLOSED | HALF_DROPPED
            });
            let cursor = get_cursor(state);
            let length = get_length(state);
            for i in 0..length {
                let i = add_cursor(cursor, i, self.buf().len());
                (*self.buf().get_unchecked(i).get()).assume_init_read();
            }
            if state & ERR_STORED != 0 {
                (*self.err().get()).assume_init_read();
            }
            if state & CLOSED == 0 && has_waker(state) {
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

impl<T, E> fmt::Debug for Receiver<T, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Receiver").finish_non_exhaustive()
    }
}

impl fmt::Display for TryNextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "ring channel is empty"),
            Self::Canceled => write!(f, "ring channel is canceled"),
        }
    }
}
