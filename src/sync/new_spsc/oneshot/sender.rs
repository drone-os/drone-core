use core::marker::PhantomData;
use core::pin::Pin;
use core::ptr::NonNull;
use core::task::{Context, Poll};
use core::{fmt, mem};

use futures::prelude::*;

use super::{
    Receiver, Shared, CLOSED, DATA_STORED, HALF_DROPPED, RX_WAKER_STORED, TX_WAKER_STORED,
};

/// The sending-half of [`oneshot::channel`](super::channel).
pub struct Sender<T> {
    pub(super) ptr: NonNull<Shared<T>>,
    phantom: PhantomData<Shared<T>>,
}

/// A future that resolves when the receiving end of a channel has hung up.
///
/// This is an `.await`-friendly interface around
/// [`poll_canceled`](Sender::poll_canceled).
#[must_use = "futures do nothing unless you `.await` or poll them"]
#[derive(Debug)]
pub struct Cancellation<'a, T> {
    sender: &'a mut Sender<T>,
}

impl<T> Sender<T> {
    pub(super) fn new(ptr: NonNull<Shared<T>>) -> Self {
        Self { ptr, phantom: PhantomData }
    }

    /// Completes this oneshot with a successful result.
    ///
    /// This function will consume `self` and indicate to the other end, the
    /// [`Receiver`], that the value provided is the result of the computation
    /// this represents.
    ///
    /// If the value is successfully enqueued for the remote end to receive,
    /// then `Ok(())` is returned. If the receiving end was dropped before this
    /// function was called, however, then `Err(value)` is returned.
    pub fn send(self, value: T) -> Result<(), T> {
        unsafe {
            let Self { ptr, .. } = self;
            mem::forget(self);
            (*ptr.as_ref().data.get()).write(value);
            let state = load_modify_state!(ptr, Acquire, AcqRel, |state| state
                | DATA_STORED
                | HALF_DROPPED);
            if state & RX_WAKER_STORED != 0 {
                let waker = (*ptr.as_ref().rx_waker.get()).assume_init_read();
                if state & HALF_DROPPED == 0 {
                    waker.wake();
                }
            }
            if state & CLOSED != 0 {
                let value = (*ptr.as_ref().data.get()).assume_init_read();
                if state & HALF_DROPPED != 0 {
                    drop(Box::from_raw(ptr.as_ptr()));
                }
                return Err(value);
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
    /// and may be able to receive a message if sent. The current task, however,
    /// is scheduled to receive a notification if the corresponding `Receiver`
    /// goes away.
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
    pub fn cancellation(&mut self) -> Cancellation<'_, T> {
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
    pub fn is_connected_to(&self, receiver: &Receiver<T>) -> bool {
        self.ptr.as_ptr() == receiver.ptr.as_ptr()
    }
}

impl<T> Drop for Sender<T> {
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

impl<T: fmt::Debug> fmt::Debug for Sender<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Sender").finish_non_exhaustive()
    }
}

impl<T> Unpin for Sender<T> {}

impl<T> Future for Cancellation<'_, T> {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        self.sender.poll_canceled(cx)
    }
}
