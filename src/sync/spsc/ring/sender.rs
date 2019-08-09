use super::{Inner, COMPLETE, INDEX_BITS, INDEX_MASK, RX_WAKER_STORED};
use crate::sync::spsc::{SpscInner, SpscInnerErr};
use alloc::sync::Arc;
use core::{
    fmt,
    pin::Pin,
    ptr,
    sync::atomic::Ordering,
    task::{Context, Poll},
};

const IS_TX_HALF: bool = true;

/// The sending-half of [`ring::channel`](super::channel).
pub struct Sender<T, E> {
    inner: Arc<Inner<T, E>>,
}

/// Error returned from [`Sender::send`](Sender::send).
#[derive(Debug)]
pub struct SendError<T> {
    /// Value which wasn't sent.
    pub value: T,
    /// The error kind.
    pub kind: SendErrorKind,
}

/// Kind of [`SendError`](SendError).
#[derive(Debug)]
pub enum SendErrorKind {
    /// The corresponding [`Receiver`](super::Receiver) is dropped.
    Canceled,
    /// Buffer overflow.
    Overflow,
}

impl<T, E> Sender<T, E> {
    #[inline]
    pub(super) fn new(inner: Arc<Inner<T, E>>) -> Self {
        Self { inner }
    }

    /// Sends a value across the channel.
    #[inline]
    pub fn send(&mut self, value: T) -> Result<(), SendError<T>> {
        self.inner.send(value)
    }

    /// Sends a value across the channel. Overwrites on overflow.
    #[inline]
    pub fn send_overwrite(&mut self, value: T) -> Result<(), T> {
        self.inner.send_overwrite(value)
    }

    /// Completes this stream with an error.
    ///
    /// If the value is successfully enqueued, then `Ok(())` is returned. If the
    /// receiving end was dropped before this function was called, then `Err` is
    /// returned with the value provided.
    #[inline]
    pub fn send_err(self, err: E) -> Result<(), E> {
        self.inner.send_err(err)
    }

    /// Polls this [`Sender`] half to detect whether the [`Receiver`] this has
    /// paired with has gone away.
    ///
    /// # Panics
    ///
    /// Like `Future::poll`, this function will panic if it's not called from
    /// within the context of a task. In other words, this should only ever be
    /// called from inside another future.
    ///
    /// If you're calling this function from a context that does not have a
    /// task, then you can use the [`is_canceled`] API instead.
    ///
    /// [`Sender`]: Sender
    /// [`Receiver`]: super::Receiver
    /// [`is_canceled`]: Sender::is_canceled
    #[inline]
    pub fn poll_cancel(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        self.inner.poll_half(
            cx,
            IS_TX_HALF,
            Ordering::Relaxed,
            Ordering::Release,
            Inner::take_cancel,
        )
    }

    /// Tests to see whether this [`Sender`]'s corresponding [`Receiver`] has
    /// gone away.
    ///
    /// [`Sender`]: Sender
    /// [`Receiver`]: super::Receiver
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
            let count = Self::get_count(*state);
            if count == self.buffer.capacity() {
                let index = self.take_index(state, count);
                Ok((*state, index))
            } else {
                let index = self.put_index(*state, count);
                Err(Some((*state, index)))
            }
        }) {
            Ok((state, index)) => {
                unsafe { ptr::drop_in_place(self.buffer.ptr().add(index)) };
                self.put(value, state, index)
            }
            Err(Some((state, index))) => self.put(value, state, index),
            Err(None) => Err(value),
        }
    }

    fn put(&self, value: T, state: usize, index: usize) -> Result<(), T> {
        let buffer_ptr = unsafe { self.buffer.ptr().add(index) };
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
                unsafe { (*self.rx_waker.get()).get_ref().wake_by_ref() };
            }
        })
        .map_err(|()| unsafe { ptr::read(buffer_ptr) })
    }

    fn put_index_try(&self, state: usize) -> Option<usize> {
        let count = Self::get_count(state);
        if count == self.buffer.capacity() {
            None
        } else {
            Some(self.put_index(state, count))
        }
    }

    fn put_index(&self, state: usize, count: usize) -> usize {
        let begin = state >> INDEX_BITS & INDEX_MASK;
        begin
            .wrapping_add(count)
            .wrapping_rem(self.buffer.capacity())
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
