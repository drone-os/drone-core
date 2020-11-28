//! A single-producer, single-consumer queue for sending values across
//! asynchronous tasks.
//!
//! See [`channel`] constructor for more.

mod receiver;
mod sender;

pub use self::{
    receiver::Receiver,
    sender::{SendError, SendErrorKind, Sender},
};

use crate::sync::spsc::{SpscInner, SpscInnerErr};
use alloc::{raw_vec::RawVec, sync::Arc};
use core::{
    cell::UnsafeCell,
    cmp,
    mem::{size_of, MaybeUninit},
    ptr, slice,
    sync::atomic::{AtomicUsize, Ordering},
    task::Waker,
};

/// Maximum capacity of the channel.
pub const MAX_CAPACITY: usize = (1 << NUMBER_BITS) - 1;

const NUMBER_MASK: usize = (1 << NUMBER_BITS) - 1;
const NUMBER_BITS: u32 = (size_of::<usize>() as u32 * 8 - OPTION_BITS) / 2;

const _RESERVED: usize = 1 << size_of::<usize>() * 8 - 1;
const COMPLETE: usize = 1 << size_of::<usize>() * 8 - 2;
const RX_WAKER_STORED: usize = 1 << size_of::<usize>() * 8 - 3;
const TX_WAKER_STORED: usize = 1 << size_of::<usize>() * 8 - 4;
const OPTION_BITS: u32 = 4;

// Layout of the state field:
//     OOOO_CCCC_LLLL
// Where O are option bits, C are cursor bits, and L are lenght bits.
//
// Cursor range: [0; MAX_CAPACITY - 1]
// Length range: [0; MAX_CAPACITY]
struct Inner<T, E> {
    state: AtomicUsize,
    buffer: RawVec<T>,
    err: UnsafeCell<Option<E>>,
    rx_waker: UnsafeCell<MaybeUninit<Waker>>,
    tx_waker: UnsafeCell<MaybeUninit<Waker>>,
}

/// Creates a new channel, returning the sender/receiver halves.
///
/// `capacity` is the capacity of the underlying ring buffer.
///
/// The [`Sender`] half is used to write values to the ring buffer. The
/// [`Receiver`] half is a [`Stream`](futures::stream::Stream) that reads the
/// values from the ring buffer.
#[inline]
pub fn channel<T, E>(capacity: usize) -> (Sender<T, E>, Receiver<T, E>) {
    let inner = Arc::new(Inner::new(capacity));
    let sender = Sender::new(Arc::clone(&inner));
    let receiver = Receiver::new(inner);
    (sender, receiver)
}

unsafe impl<T: Send, E: Send> Send for Inner<T, E> {}
unsafe impl<T: Send, E: Send> Sync for Inner<T, E> {}

impl<T, E> Inner<T, E> {
    #[inline]
    fn new(capacity: usize) -> Self {
        assert!(capacity <= MAX_CAPACITY);
        Self {
            state: AtomicUsize::new(0),
            buffer: RawVec::with_capacity(capacity),
            err: UnsafeCell::new(None),
            rx_waker: UnsafeCell::new(MaybeUninit::zeroed()),
            tx_waker: UnsafeCell::new(MaybeUninit::zeroed()),
        }
    }
}

impl<T, E> Drop for Inner<T, E> {
    fn drop(&mut self) {
        let state = self.state_load(Ordering::Acquire);
        let length = state & NUMBER_MASK;
        let cursor = state >> NUMBER_BITS & NUMBER_MASK;
        let end = cursor.wrapping_add(length).wrapping_rem(self.buffer.capacity());
        match cursor.cmp(&end) {
            cmp::Ordering::Equal => unsafe {
                ptr::drop_in_place(slice::from_raw_parts_mut(
                    self.buffer.ptr(),
                    self.buffer.capacity(),
                ));
            },
            cmp::Ordering::Less => unsafe {
                ptr::drop_in_place(slice::from_raw_parts_mut(
                    self.buffer.ptr().add(cursor),
                    end - cursor,
                ));
            },
            cmp::Ordering::Greater => unsafe {
                ptr::drop_in_place(slice::from_raw_parts_mut(self.buffer.ptr(), end));
                ptr::drop_in_place(slice::from_raw_parts_mut(
                    self.buffer.ptr().add(cursor),
                    self.buffer.capacity() - cursor,
                ));
            },
        }
    }
}

impl<T, E> SpscInner<AtomicUsize, usize> for Inner<T, E> {
    const COMPLETE: usize = COMPLETE;
    const RX_WAKER_STORED: usize = RX_WAKER_STORED;
    const TX_WAKER_STORED: usize = TX_WAKER_STORED;
    const ZERO: usize = 0;

    #[inline]
    fn state_load(&self, order: Ordering) -> usize {
        self.state.load(order)
    }

    #[inline]
    fn compare_exchange_weak(
        &self,
        current: usize,
        new: usize,
        success: Ordering,
        failure: Ordering,
    ) -> Result<usize, usize> {
        self.state.compare_exchange_weak(current, new, success, failure)
    }

    #[inline]
    unsafe fn rx_waker_mut(&self) -> &mut MaybeUninit<Waker> {
        unsafe { &mut *self.rx_waker.get() }
    }

    #[inline]
    unsafe fn tx_waker_mut(&self) -> &mut MaybeUninit<Waker> {
        unsafe { &mut *self.tx_waker.get() }
    }
}

impl<T, E> SpscInnerErr<AtomicUsize, usize> for Inner<T, E> {
    type Error = E;

    unsafe fn err_mut(&self) -> &mut Option<Self::Error> {
        unsafe { &mut *self.err.get() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::{
        pin::Pin,
        sync::atomic::AtomicUsize,
        task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
    };
    use futures::stream::Stream;

    struct Counter(AtomicUsize);

    impl Counter {
        fn to_waker(&'static self) -> Waker {
            unsafe fn clone(counter: *const ()) -> RawWaker {
                RawWaker::new(counter, &VTABLE)
            }
            unsafe fn wake(counter: *const ()) {
                unsafe { (*(counter as *const Counter)).0.fetch_add(1, Ordering::SeqCst) };
            }
            static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake, drop);
            unsafe { Waker::from_raw(RawWaker::new(self as *const _ as *const (), &VTABLE)) }
        }
    }

    #[test]
    fn send_sync() {
        static COUNTER: Counter = Counter(AtomicUsize::new(0));
        let (mut tx, mut rx) = channel::<usize, ()>(10);
        assert_eq!(tx.send(314).unwrap(), ());
        drop(tx);
        let waker = COUNTER.to_waker();
        let mut cx = Context::from_waker(&waker);
        COUNTER.0.store(0, Ordering::SeqCst);
        assert_eq!(Pin::new(&mut rx).poll_next(&mut cx), Poll::Ready(Some(Ok(314))));
        assert_eq!(Pin::new(&mut rx).poll_next(&mut cx), Poll::Ready(None));
        assert_eq!(COUNTER.0.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn send_async() {
        static COUNTER: Counter = Counter(AtomicUsize::new(0));
        let (mut tx, mut rx) = channel::<usize, ()>(10);
        let waker = COUNTER.to_waker();
        let mut cx = Context::from_waker(&waker);
        COUNTER.0.store(0, Ordering::SeqCst);
        assert_eq!(Pin::new(&mut rx).poll_next(&mut cx), Poll::Pending);
        assert_eq!(tx.send(314).unwrap(), ());
        assert_eq!(Pin::new(&mut rx).poll_next(&mut cx), Poll::Ready(Some(Ok(314))));
        assert_eq!(Pin::new(&mut rx).poll_next(&mut cx), Poll::Pending);
        drop(tx);
        assert_eq!(Pin::new(&mut rx).poll_next(&mut cx), Poll::Ready(None));
        assert_eq!(COUNTER.0.load(Ordering::SeqCst), 2);
    }
}
