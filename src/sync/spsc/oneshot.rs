//! A channel for sending a single message between asynchronous tasks.
//!
//! See [`channel`](oneshot::channel) constructor for more.

mod receiver;
mod sender;

pub use self::{
    receiver::{Canceled, Receiver},
    sender::Sender,
};

use crate::sync::spsc::SpscInner;
use alloc::sync::Arc;
use core::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    sync::atomic::{AtomicU8, Ordering},
    task::Waker,
};

#[allow(clippy::identity_op)]
const TX_WAKER_STORED: u8 = 1 << 0;
const RX_WAKER_STORED: u8 = 1 << 1;
const COMPLETE: u8 = 1 << 2;

struct Inner<T> {
    state: AtomicU8,
    data: UnsafeCell<Option<T>>,
    rx_waker: UnsafeCell<MaybeUninit<Waker>>,
    tx_waker: UnsafeCell<MaybeUninit<Waker>>,
}

/// Creates a new one-shot channel, returning the sender/receiver halves.
///
/// The [`Sender`] half is used to signal the end of a computation and provide
/// its value. The [`Receiver`] half is a [`Future`](core::future::Future)
/// resolving to the value that was given to the [`Sender`] half.
#[inline]
pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let inner = Arc::new(Inner::new());
    let sender = Sender::new(Arc::clone(&inner));
    let receiver = Receiver::new(inner);
    (sender, receiver)
}

unsafe impl<T: Send> Send for Inner<T> {}
unsafe impl<T: Send> Sync for Inner<T> {}

impl<T> Inner<T> {
    #[inline]
    fn new() -> Self {
        Self {
            state: AtomicU8::new(0),
            data: UnsafeCell::new(None),
            rx_waker: UnsafeCell::new(MaybeUninit::zeroed()),
            tx_waker: UnsafeCell::new(MaybeUninit::zeroed()),
        }
    }
}

impl<T> SpscInner<AtomicU8, u8> for Inner<T> {
    const ZERO: u8 = 0;
    const RX_WAKER_STORED: u8 = RX_WAKER_STORED;
    const TX_WAKER_STORED: u8 = TX_WAKER_STORED;
    const COMPLETE: u8 = COMPLETE;

    #[inline]
    fn state_load(&self, order: Ordering) -> u8 {
        self.state.load(order)
    }

    #[inline]
    fn compare_exchange_weak(
        &self,
        current: u8,
        new: u8,
        success: Ordering,
        failure: Ordering,
    ) -> Result<u8, u8> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use core::{
        future::Future,
        pin::Pin,
        sync::atomic::AtomicUsize,
        task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
    };

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
        let (tx, mut rx) = channel::<usize>();
        assert_eq!(tx.send(314), Ok(()));
        let waker = COUNTER.to_waker();
        let mut cx = Context::from_waker(&waker);
        COUNTER.0.store(0, Ordering::SeqCst);
        assert_eq!(Pin::new(&mut rx).poll(&mut cx), Poll::Ready(Ok(314)));
        assert_eq!(COUNTER.0.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn send_async() {
        static COUNTER: Counter = Counter(AtomicUsize::new(0));
        let (tx, mut rx) = channel::<usize>();
        let waker = COUNTER.to_waker();
        let mut cx = Context::from_waker(&waker);
        COUNTER.0.store(0, Ordering::SeqCst);
        assert_eq!(Pin::new(&mut rx).poll(&mut cx), Poll::Pending);
        assert_eq!(tx.send(314), Ok(()));
        assert_eq!(Pin::new(&mut rx).poll(&mut cx), Poll::Ready(Ok(314)));
        assert_eq!(COUNTER.0.load(Ordering::SeqCst), 1);
    }
}
