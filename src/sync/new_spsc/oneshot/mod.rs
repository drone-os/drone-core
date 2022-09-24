//! A channel for sending a single message between asynchronous tasks.
//!
//! This is a single-producer, single-consumer channel.
//!
//! # Memory footprint
//!
//! Call to [`channel`] creates one allocation of an inner shared object. Each
//! returned half is a word-sized pointer to the shared object.
//!
//! The shared object consists of a the generic type `T`, byte-sized state
//! field, and two double-word-sized [`Waker`] objects.
//!
//! # State field structure
//!
//! Channel state is an atomic `u8` value, initially zeroed, with the following
//! structure:
//!
//! `000HCDRT`
//!
//! Where the bit, if set, indicates:
//! * `T` - [`Sender`] half waker is stored
//! * `R` - [`Receiver`] half waker is stored
//! * `D` - data value of type `T` is stored
//! * `C` - [`Receiver`] half is closed
//! * `H` - one of the halves was dropped
//! * `0` - ignored

mod receiver;
mod sender;

use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::ptr::NonNull;
#[cfg(feature = "atomics")]
use core::sync::atomic::AtomicU8;
use core::task::Waker;

pub use self::receiver::{Canceled, Receiver};
pub use self::sender::Sender;
#[cfg(not(feature = "atomics"))]
use crate::sync::soft_atomic::Atomic;

/// Creates a new one-shot channel, returning the sender/receiver halves.
///
/// The [`Sender`] half is used to signal the end of a computation and provide
/// its value. The [`Receiver`] half is a [`Future`](core::future::Future)
/// resolving to the value that was given to the [`Sender`] half.
///
/// See [the module-level documentation](self) for details.
pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let shared = unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(Shared::new()))) };
    let sender = Sender::new(shared);
    let receiver = Receiver::new(shared);
    (sender, receiver)
}

#[allow(clippy::identity_op)]
const TX_WAKER_STORED: u8 = 1 << 0;
const RX_WAKER_STORED: u8 = 1 << 1;
const DATA_STORED: u8 = 1 << 2;
const CLOSED: u8 = 1 << 3;
const HALF_DROPPED: u8 = 1 << 4;

struct Shared<T> {
    #[cfg(not(feature = "atomics"))]
    state: Atomic<u8>,
    #[cfg(feature = "atomics")]
    state: AtomicU8,
    data: UnsafeCell<MaybeUninit<T>>,
    rx_waker: UnsafeCell<MaybeUninit<Waker>>,
    tx_waker: UnsafeCell<MaybeUninit<Waker>>,
}

impl<T> Shared<T> {
    fn new() -> Self {
        Self {
            #[cfg(not(feature = "atomics"))]
            state: Atomic::new(0),
            #[cfg(feature = "atomics")]
            state: AtomicU8::new(0),
            data: UnsafeCell::new(MaybeUninit::uninit()),
            rx_waker: UnsafeCell::new(MaybeUninit::uninit()),
            tx_waker: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }
}

#[cfg(test)]
mod tests {
    use core::future::Future;
    use core::pin::Pin;
    use core::sync::atomic::AtomicUsize;
    use core::sync::atomic::Ordering::SeqCst;
    use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

    use super::*;

    struct Counter(AtomicUsize);

    impl Counter {
        fn to_waker(&'static self) -> Waker {
            unsafe fn clone(counter: *const ()) -> RawWaker {
                RawWaker::new(counter, &VTABLE)
            }
            unsafe fn wake(counter: *const ()) {
                unsafe { (*(counter as *const Counter)).0.fetch_add(1, SeqCst) };
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
        COUNTER.0.store(0, SeqCst);
        assert_eq!(Pin::new(&mut rx).poll(&mut cx), Poll::Ready(Ok(314)));
        assert_eq!(COUNTER.0.load(SeqCst), 0);
    }

    #[test]
    fn send_async() {
        static COUNTER: Counter = Counter(AtomicUsize::new(0));
        let (tx, mut rx) = channel::<usize>();
        let waker = COUNTER.to_waker();
        let mut cx = Context::from_waker(&waker);
        COUNTER.0.store(0, SeqCst);
        assert_eq!(Pin::new(&mut rx).poll(&mut cx), Poll::Pending);
        assert_eq!(tx.send(314), Ok(()));
        assert_eq!(Pin::new(&mut rx).poll(&mut cx), Poll::Ready(Ok(314)));
        assert_eq!(COUNTER.0.load(SeqCst), 1);
    }
}
