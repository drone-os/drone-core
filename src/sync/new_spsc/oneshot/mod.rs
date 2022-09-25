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
//! `00HWCDRT`
//!
//! Where the bit, if set, indicates:
//! * `T` - [`Sender`] half waker is stored
//! * `R` - [`Receiver`] half waker is stored
//! * `D` - data value of type `T` is stored
//! * `C` - [`Receiver`] half is closed
//! * `W` - [`Receiver`] half is closed, but there is pending data
//! * `H` - one of the halves was dropped
//! * `0` - ignored

mod receiver;
mod sender;

use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::ptr::NonNull;
#[cfg(feature = "_atomics")]
use core::sync::atomic::AtomicU8;
use core::task::Waker;

#[cfg(loom)]
use loom::sync::atomic::AtomicU8;

pub use self::receiver::{Canceled, Receiver};
pub use self::sender::{Cancellation, Sender};
#[cfg(not(any(feature = "_atomics", loom)))]
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

const TX_WAKER_STORED_SHIFT: u8 = 0;
const RX_WAKER_STORED_SHIFT: u8 = 1;
const DATA_STORED_SHIFT: u8 = 2;
const CLOSED_SHIFT: u8 = 3;
const CLOSED_WITH_DATA_SHIFT: u8 = 4;
const HALF_DROPPED_SHIFT: u8 = 5;

const TX_WAKER_STORED: u8 = 1 << TX_WAKER_STORED_SHIFT;
const RX_WAKER_STORED: u8 = 1 << RX_WAKER_STORED_SHIFT;
const DATA_STORED: u8 = 1 << DATA_STORED_SHIFT;
const CLOSED: u8 = 1 << CLOSED_SHIFT;
const CLOSED_WITH_DATA: u8 = 1 << CLOSED_WITH_DATA_SHIFT;
const HALF_DROPPED: u8 = 1 << HALF_DROPPED_SHIFT;

impl<T> Unpin for Sender<T> {}
impl<T> Unpin for Receiver<T> {}
unsafe impl<T: Send> Send for Sender<T> {}
unsafe impl<T: Send> Sync for Receiver<T> {}

struct Shared<T> {
    #[cfg(not(any(feature = "_atomics", loom)))]
    state: Atomic<u8>,
    #[cfg(any(feature = "_atomics", loom))]
    state: AtomicU8,
    data: UnsafeCell<MaybeUninit<T>>,
    rx_waker: UnsafeCell<MaybeUninit<Waker>>,
    tx_waker: UnsafeCell<MaybeUninit<Waker>>,
}

impl<T> Shared<T> {
    fn new() -> Self {
        Self {
            #[cfg(not(any(feature = "_atomics", loom)))]
            state: Atomic::new(0),
            #[cfg(any(feature = "_atomics", loom))]
            state: AtomicU8::new(0),
            data: UnsafeCell::new(MaybeUninit::uninit()),
            rx_waker: UnsafeCell::new(MaybeUninit::uninit()),
            tx_waker: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }
}
