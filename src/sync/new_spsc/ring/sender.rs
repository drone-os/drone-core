use core::cell::UnsafeCell;
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::pin::Pin;
use core::ptr::NonNull;
use core::task::{Context, Poll, Waker};
use core::{fmt, mem};

use futures::prelude::*;

use super::{
    add_cursor, add_length, claim_next_if_full, get_cursor, get_length, has_flush_waker,
    has_ready_waker, has_waker, set_flush_waker, set_ready_waker, Receiver, Shared, State, CLOSED,
    ERR_STORED, HALF_DROPPED, RX_WAKER_STORED,
};

/// The sending-half of [`ring::channel`](super::channel).
pub struct Sender<T, E> {
    pub(super) ptr: NonNull<Shared<T, E>>,
    phantom: PhantomData<Shared<T, E>>,
}

/// This enumeration is the list of the possible reasons why [`Receiver`] could
/// not send data.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SendError {
    /// The data could not be sent on the channel because the channel's internal
    /// ring buffer is full.
    Full,
    /// The corresponding [`Receiver`] is closed or dropped.
    Canceled,
}

/// The error type returned from [`Sender::try_send`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TrySendError<T> {
    /// The reason why [`Sender::try_send`] could not send data.
    pub err: SendError,
    /// The value provided to the failed [`Sender::try_send`] call.
    pub value: T,
}

impl<T, E> Sender<T, E> {
    pub(super) fn new(ptr: NonNull<Shared<T, E>>) -> Self {
        Self { ptr, phantom: PhantomData }
    }

    /// Attempts to send a message on this `Sender`, returning the message if
    /// there was an error.
    pub fn try_send(&mut self, value: T) -> Result<(), TrySendError<T>> {
        unsafe {
            let mut state = load_atomic!(self.state(), Relaxed);
            let length = get_length(state);
            if length == self.buf().len() {
                return Err(TrySendError { err: SendError::Full, value });
            }
            let index = add_cursor(get_cursor(state), length, self.buf().len());
            (*self.buf().get_unchecked(index).get()).write(value);
            state = modify_atomic!(self.state(), Acquire, AcqRel, |state| add_length(state, 1));
            if state & CLOSED != 0 {
                let value = (*self.buf().get_unchecked(index).get()).assume_init_read();
                return Err(TrySendError { err: SendError::Canceled, value });
            }
            if state & RX_WAKER_STORED != 0 {
                (*self.rx_waker().get()).assume_init_ref().wake_by_ref();
            }
            Ok(())
        }
    }

    /// Sends a message on this channel, overwriting the oldest value stored in
    /// the ring buffer if it's full. If the receiving end was dropped before
    /// this function was called, then `Err` is returned with the value
    /// provided.
    pub fn send_overwrite(&mut self, value: T) -> Result<(), T> {
        unsafe {
            let mut state = load_atomic!(self.state(), Relaxed);
            let mut overwrite = false;
            let mut length = get_length(state);
            if length == self.buf().len() {
                state = modify_atomic!(self.state(), Relaxed, Relaxed, |state| {
                    claim_next_if_full(state, self.buf().len(), &mut overwrite)
                });
                length = get_length(state);
            }
            let mut index = get_cursor(state);
            if overwrite {
                (*self.buf().get_unchecked(index).get()).assume_init_read();
            } else {
                index = add_cursor(index, length, self.buf().len());
            }
            (*self.buf().get_unchecked(index).get()).write(value);
            state = modify_atomic!(self.state(), Acquire, AcqRel, |state| add_length(state, 1));
            if state & CLOSED != 0 {
                return Err((*self.buf().get_unchecked(index).get()).assume_init_read());
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
            (*ptr.as_ref().hdr.err.get()).write(err);
            let state = load_modify_atomic!(ptr.as_ref().hdr.state, Acquire, AcqRel, |state| state
                | if state & CLOSED == 0 { ERR_STORED } else { 0 }
                | HALF_DROPPED);
            if state & RX_WAKER_STORED != 0 {
                let waker = (*ptr.as_ref().hdr.rx_waker.get()).assume_init_read();
                if state & CLOSED == 0 {
                    waker.wake();
                }
            }
            if state & CLOSED != 0 {
                let err = (*ptr.as_ref().hdr.err.get()).assume_init_read();
                if state & HALF_DROPPED != 0 {
                    drop(Box::from_raw(ptr.as_ptr()));
                }
                return Err(err);
            }
            Ok(())
        }
    }

    /// Tests to see whether this `Sender`'s corresponding `Receiver` has been
    /// dropped.
    pub fn is_canceled(&self) -> bool {
        unsafe {
            let state = load_atomic!(self.state(), Relaxed);
            state & CLOSED != 0
        }
    }

    /// Tests to see whether this `Sender` is connected to the given `Receiver`.
    /// That is, whether they were created by the same call to `channel`.
    pub fn is_connected_to(&self, receiver: &Receiver<T, E>) -> bool {
        self.ptr.as_ptr() == receiver.ptr.as_ptr()
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

    unsafe fn buf(&self) -> &[UnsafeCell<MaybeUninit<T>>] {
        unsafe { &self.ptr.as_ref().buf }
    }
}

impl<T, E> Sink<T> for Sender<T, E> {
    type Error = SendError;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        unsafe {
            let mut state = load_atomic!(self.state(), Relaxed);
            if state & CLOSED != 0 {
                return Poll::Ready(Err(SendError::Canceled));
            }
            if get_length(state) < self.buf().len() {
                return Poll::Ready(Ok(()));
            }
            if !has_ready_waker(state) {
                let write_waker = !has_waker(state);
                if write_waker {
                    (*self.tx_waker().get()).write(cx.waker().clone());
                }
                state =
                    modify_atomic!(self.state(), Relaxed, Release, |state| set_ready_waker(state));
                if state & HALF_DROPPED != 0 {
                    if write_waker {
                        (*self.tx_waker().get()).assume_init_read();
                    }
                    return Poll::Ready(Err(SendError::Canceled));
                }
                if get_length(state) < self.buf().len() {
                    return Poll::Ready(Ok(()));
                }
            }
            Poll::Pending
        }
    }

    fn start_send(mut self: Pin<&mut Self>, value: T) -> Result<(), Self::Error> {
        self.try_send(value).map_err(|e| e.err)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        unsafe {
            let mut state = load_atomic!(self.state(), Relaxed);
            if get_length(state) == 0 {
                return Poll::Ready(Ok(()));
            }
            if !has_flush_waker(state) {
                let write_waker = !has_waker(state);
                if write_waker {
                    (*self.tx_waker().get()).write(cx.waker().clone());
                }
                state =
                    modify_atomic!(self.state(), Relaxed, Release, |state| set_flush_waker(state));
                if get_length(state) == 0 {
                    if write_waker && state & HALF_DROPPED != 0 {
                        (*self.tx_waker().get()).assume_init_read();
                    }
                    return Poll::Ready(Ok(()));
                }
            }
            Poll::Pending
        }
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.poll_flush(cx)
    }
}

impl<T, E> Drop for Sender<T, E> {
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

impl<T, E> fmt::Debug for Sender<T, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Sender").finish_non_exhaustive()
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

impl<T> fmt::Display for TrySendError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.err, f)
    }
}
