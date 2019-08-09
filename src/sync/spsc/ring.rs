//! A single-producer, single-consumer channel based on a ring buffer.
//!
//! See [`ring::channel`] documentation for more details.

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
    mem::{self, MaybeUninit},
    ptr, slice,
    sync::atomic::{AtomicUsize, Ordering},
    task::Waker,
};

/// Maximum capacity of the channel.
pub const MAX_CAPACITY: usize = (1 << INDEX_BITS) - 1;

const INDEX_MASK: usize = (1 << INDEX_BITS) - 1;
const INDEX_BITS: usize = (mem::size_of::<usize>() * 8 - LOCK_BITS) / 2;
const LOCK_BITS: usize = 4;
const _RESERVED: usize = 1 << mem::size_of::<usize>() * 8 - 1;
const COMPLETE: usize = 1 << mem::size_of::<usize>() * 8 - 2;
const RX_WAKER_STORED: usize = 1 << mem::size_of::<usize>() * 8 - 3;
const TX_WAKER_STORED: usize = 1 << mem::size_of::<usize>() * 8 - 4;

// Layout of the state field:
//     LLLL_BBBB_CCCC
// Where L is lock bits, B is begin bits, and C is count bits.
//
// Begin range: [0; MAX_CAPACITY - 1]
// Count range: [0; MAX_CAPACITY]
struct Inner<T, E> {
    state: AtomicUsize,
    buffer: RawVec<T>,
    err: UnsafeCell<Option<E>>,
    rx_waker: UnsafeCell<MaybeUninit<Waker>>,
    tx_waker: UnsafeCell<MaybeUninit<Waker>>,
}

/// Creates a new asynchronous channel, returning the receiver/sender halves.
/// All data sent on the [`Sender`] will become available on the [`Receiver`] in
/// the same order as it was sent.
///
/// Only one [`Receiver`]/[`Sender`] is supported.
///
/// [`Receiver`]: Receiver
/// [`Sender`]: Sender
#[inline]
pub fn channel<T, E>(capacity: usize) -> (Receiver<T, E>, Sender<T, E>) {
    let inner = Arc::new(Inner::new(capacity));
    let receiver = Receiver::new(Arc::clone(&inner));
    let sender = Sender::new(inner);
    (receiver, sender)
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
        let count = state & INDEX_MASK;
        let begin = state >> INDEX_BITS & INDEX_MASK;
        let end = begin
            .wrapping_add(count)
            .wrapping_rem(self.buffer.capacity());
        match begin.cmp(&end) {
            cmp::Ordering::Equal => unsafe {
                ptr::drop_in_place(slice::from_raw_parts_mut(
                    self.buffer.ptr(),
                    self.buffer.capacity(),
                ));
            },
            cmp::Ordering::Less => unsafe {
                ptr::drop_in_place(slice::from_raw_parts_mut(
                    self.buffer.ptr().add(begin),
                    end - begin,
                ));
            },
            cmp::Ordering::Greater => unsafe {
                ptr::drop_in_place(slice::from_raw_parts_mut(self.buffer.ptr(), end));
                ptr::drop_in_place(slice::from_raw_parts_mut(
                    self.buffer.ptr().add(begin),
                    self.buffer.capacity() - begin,
                ));
            },
        }
    }
}

impl<T, E> SpscInner<AtomicUsize, usize> for Inner<T, E> {
    const ZERO: usize = 0;
    const RX_WAKER_STORED: usize = RX_WAKER_STORED;
    const TX_WAKER_STORED: usize = TX_WAKER_STORED;
    const COMPLETE: usize = COMPLETE;

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
        self.state
            .compare_exchange_weak(current, new, success, failure)
    }

    #[inline]
    unsafe fn rx_waker_mut(&self) -> &mut MaybeUninit<Waker> {
        &mut *self.rx_waker.get()
    }

    #[inline]
    unsafe fn tx_waker_mut(&self) -> &mut MaybeUninit<Waker> {
        &mut *self.tx_waker.get()
    }
}

impl<T, E> SpscInnerErr<AtomicUsize, usize> for Inner<T, E> {
    type Error = E;

    unsafe fn err_mut(&self) -> &mut Option<Self::Error> {
        &mut *self.err.get()
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
                (*(counter as *const Counter))
                    .0
                    .fetch_add(1, Ordering::SeqCst);
            }
            static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake, drop);
            unsafe { Waker::from_raw(RawWaker::new(self as *const _ as *const (), &VTABLE)) }
        }
    }

    #[test]
    fn send_sync() {
        static COUNTER: Counter = Counter(AtomicUsize::new(0));
        let (mut rx, mut tx) = channel::<usize, ()>(10);
        assert_eq!(tx.send(314).unwrap(), ());
        drop(tx);
        let waker = COUNTER.to_waker();
        let mut cx = Context::from_waker(&waker);
        COUNTER.0.store(0, Ordering::SeqCst);
        assert_eq!(
            Pin::new(&mut rx).poll_next(&mut cx),
            Poll::Ready(Some(Ok(314)))
        );
        assert_eq!(Pin::new(&mut rx).poll_next(&mut cx), Poll::Ready(None));
        assert_eq!(COUNTER.0.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn send_async() {
        static COUNTER: Counter = Counter(AtomicUsize::new(0));
        let (mut rx, mut tx) = channel::<usize, ()>(10);
        let waker = COUNTER.to_waker();
        let mut cx = Context::from_waker(&waker);
        COUNTER.0.store(0, Ordering::SeqCst);
        assert_eq!(Pin::new(&mut rx).poll_next(&mut cx), Poll::Pending);
        assert_eq!(tx.send(314).unwrap(), ());
        assert_eq!(
            Pin::new(&mut rx).poll_next(&mut cx),
            Poll::Ready(Some(Ok(314)))
        );
        assert_eq!(Pin::new(&mut rx).poll_next(&mut cx), Poll::Pending);
        drop(tx);
        assert_eq!(Pin::new(&mut rx).poll_next(&mut cx), Poll::Ready(None));
        assert_eq!(COUNTER.0.load(Ordering::SeqCst), 2);
    }
}
