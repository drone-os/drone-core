use core::cell::UnsafeCell;
use core::fmt;
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::pin::Pin;
use core::ptr::NonNull;
use core::task::{Context, Poll, Waker};

use futures::future::FusedFuture;
use futures::prelude::*;

use super::{Shared, State, CLOSED, DATA_STORED, HALF_DROPPED, RX_WAKER_STORED, TX_WAKER_STORED};

/// The receiving-half of [`oneshot::channel`](super::channel).
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Receiver<T> {
    pub(super) ptr: NonNull<Shared<T>>,
    phantom: PhantomData<Shared<T>>,
}

/// Error returned from a [`Receiver`] when the corresponding
/// [`Sender`](super::Sender) is dropped.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Canceled;

impl<T> Receiver<T> {
    pub(super) fn new(ptr: NonNull<Shared<T>>) -> Self {
        Self { ptr, phantom: PhantomData }
    }

    /// Gracefully close this receiver, preventing any subsequent attempts to
    /// send to it.
    ///
    /// Any `send` operation which happens after this method returns is
    /// guaranteed to fail. After calling this method, you can use
    /// [`Receiver::poll`](Future::poll) to determine whether a message had
    /// previously been sent.
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

    /// Attempts to receive a message outside of the context of a task.
    ///
    /// Does not schedule a task wakeup or have any other side effects.
    ///
    /// A return value of `Ok(None)` must be considered immediately stale (out
    /// of date) unless [`close`](Receiver::close) has been called first.
    ///
    /// Returns an error if the sender was dropped.
    pub fn try_recv(&mut self) -> Result<Option<T>, Canceled> {
        unsafe {
            let state =
                load_modify_atomic!(self.state(), Relaxed, Acquire, |state| state & !DATA_STORED);
            if state & DATA_STORED != 0 {
                return Ok(Some((*self.data().get()).assume_init_read()));
            }
            if state & HALF_DROPPED != 0 || state & CLOSED != 0 {
                return Err(Canceled);
            }
            Ok(None)
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

    unsafe fn data(&self) -> &UnsafeCell<MaybeUninit<T>> {
        unsafe { &self.ptr.as_ref().data }
    }
}

impl<T> Future for Receiver<T> {
    type Output = Result<T, Canceled>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        unsafe {
            let mut state =
                load_modify_atomic!(self.state(), Relaxed, Acquire, |state| state & !DATA_STORED);
            if state & DATA_STORED != 0 {
                return Poll::Ready(Ok((*self.data().get()).assume_init_read()));
            }
            if state & HALF_DROPPED != 0 || state & CLOSED != 0 {
                return Poll::Ready(Err(Canceled));
            }
            if state & RX_WAKER_STORED == 0 {
                (*self.rx_waker().get()).write(cx.waker().clone());
                state = modify_atomic!(self.state(), Acquire, AcqRel, |state| state & !DATA_STORED
                    | RX_WAKER_STORED);
                if state & HALF_DROPPED != 0 {
                    (*self.rx_waker().get()).assume_init_read();
                    if state & DATA_STORED != 0 {
                        return Poll::Ready(Ok((*self.data().get()).assume_init_read()));
                    }
                    return Poll::Ready(Err(Canceled));
                }
            }
            Poll::Pending
        }
    }
}

impl<T> FusedFuture for Receiver<T> {
    fn is_terminated(&self) -> bool {
        unsafe {
            let state = load_atomic!(self.state(), Relaxed);
            (state & HALF_DROPPED != 0 || state & CLOSED != 0) && state & DATA_STORED == 0
        }
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        unsafe {
            let state = load_modify_atomic!(self.state(), Relaxed, Acquire, |state| state
                | CLOSED
                | HALF_DROPPED);
            if state & DATA_STORED != 0 {
                (*self.data().get()).assume_init_read();
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

impl<T> fmt::Debug for Receiver<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Receiver").finish_non_exhaustive()
    }
}

impl fmt::Display for Canceled {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "oneshot canceled")
    }
}
