use alloc::sync::Arc;
use core::sync::atomic::Ordering;
use core::task::{Context, Poll};
use core::{fmt, ptr};

use super::{Inner, COMPLETE, NUMBER_BITS, NUMBER_MASK, RX_WAKER_STORED};
use crate::sync::spsc::{SpscInner, SpscInnerErr};

const IS_TX_HALF: bool = true;

/// The sending-half of [`ring::channel`](super::channel).
pub struct Sender<T, E> {
    inner: Arc<Inner<T, E>>,
}

/// The error type returned from [`Sender::send`].
#[derive(Debug)]
pub struct SendError<T> {
    /// The value which has been not sent.
    pub value: T,
    /// The error kind.
    pub kind: SendErrorKind,
}

/// Kind of [`SendError`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SendErrorKind {
    /// The corresponding [`Receiver`](super::Receiver) is dropped.
    Canceled,
    /// The ring buffer overflow.
    Overflow,
}

impl<T, E> Sender<T, E> {
    pub(super) fn new(inner: Arc<Inner<T, E>>) -> Self {
        Self { inner }
    }

    /// Puts `value` to the ring buffer. The value can be immediately read by
    /// the receiving half.
    ///
    /// If the value is successfully enqueued for the remote end to receive,
    /// then `Ok(())` is returned. However if the receiving end was dropped
    /// before this function was called or there is the ring buffer overflow,
    /// then `Err` is returned with the value provided.
    #[inline]
    pub fn send(&mut self, value: T) -> Result<(), SendError<T>> {
        self.inner.send(value)
    }

    /// Puts `value` to the ring buffer. The value can be immediately read by
    /// the receiving half. This method overwrites old items on overflow.
    ///
    /// If the value is successfully enqueued for the remote end to receive,
    /// then `Ok(())` is returned. If the receiving end was dropped before this
    /// function was called, however, then `Err` is returned with the value
    /// provided.
    #[inline]
    pub fn send_overwrite(&mut self, value: T) -> Result<(), T> {
        self.inner.send_overwrite(value)
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
    /// alive and may be able to receive values if sent. The current task,
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

impl<T, E> Drop for Sender<T, E> {
    #[inline]
    fn drop(&mut self) {
        self.inner.close_half(IS_TX_HALF);
    }
}

impl<T, E> Inner<T, E> {
    #[allow(clippy::option_if_let_else)]
    fn send(&self, value: T) -> Result<(), SendError<T>> {
        let state = self.state_load(Ordering::Acquire);
        if let Some(index) = self.put_index_try(state) {
            self.put(value, state, index)
                .map_err(|value| SendError::new(value, SendErrorKind::Canceled))
        } else {
            Err(SendError::new(value, SendErrorKind::Overflow))
        }
    }

    fn send_overwrite(&self, value: T) -> Result<(), T> {
        let state = self.state_load(Ordering::Acquire);
        if let Some(index) = self.put_index_try(state) {
            return self.put(value, state, index);
        }
        match self.transaction(state, Ordering::Acquire, Ordering::Acquire, |state| {
            if *state & COMPLETE != 0 {
                return Err(None);
            }
            let length = Self::get_length(*state);
            if length == self.capacity {
                let index = self.take_index(state, length);
                Ok((*state, index))
            } else {
                let index = self.put_index(*state, length);
                Err(Some((*state, index)))
            }
        }) {
            Ok((state, index)) => {
                unsafe { ptr::drop_in_place(self.ptr.as_ptr().add(index)) };
                self.put(value, state, index)
            }
            Err(Some((state, index))) => self.put(value, state, index),
            Err(None) => Err(value),
        }
    }

    fn put(&self, value: T, state: usize, index: usize) -> Result<(), T> {
        let buffer_ptr = unsafe { self.ptr.as_ptr().add(index) };
        unsafe { ptr::write(buffer_ptr, value) };
        self.transaction(state, Ordering::AcqRel, Ordering::Acquire, |state| {
            if *state & COMPLETE == 0 {
                *state = state.wrapping_add(1);
                Ok(*state)
            } else {
                Err(())
            }
        })
        .map(|state| {
            if state & RX_WAKER_STORED != 0 {
                unsafe { (*self.rx_waker.get()).assume_init_ref().wake_by_ref() };
            }
        })
        .map_err(|()| unsafe { ptr::read(buffer_ptr) })
    }

    fn put_index_try(&self, state: usize) -> Option<usize> {
        let length = Self::get_length(state);
        if length == self.capacity { None } else { Some(self.put_index(state, length)) }
    }

    fn put_index(&self, state: usize, length: usize) -> usize {
        let cursor = state >> NUMBER_BITS & NUMBER_MASK;
        cursor.wrapping_add(length).wrapping_rem(self.capacity)
    }
}

impl<T> SendError<T> {
    #[inline]
    fn new(value: T, kind: SendErrorKind) -> Self {
        Self { value, kind }
    }
}

impl<T: fmt::Display> fmt::Display for SendError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.kind.fmt(f)
    }
}

impl fmt::Display for SendErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SendErrorKind::Canceled => write!(f, "Receiver is dropped."),
            SendErrorKind::Overflow => write!(f, "Channel buffer overflow."),
        }
    }
}
