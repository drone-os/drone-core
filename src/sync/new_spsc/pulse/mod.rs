//! A channel for sending `()` values (pulses) across asynchronous tasks. An
//! optimized version of [`ring::channel<(), E>`](super::ring::channel).
//!
//! This is a single-producer, single-consumer channel.
//!
//! # Memory footprint
//!
//! Call to [`channel`] creates one allocation of an inner shared object. Each
//! returned half is a word-sized pointer to the shared object.
//!
//! The shared object consists of a the generic type `E`, word-sized state
//! field, and two double-word-sized [`Waker`] objects.
//!
//! # State field structure
//!
//! Channel state is an atomic `usize` value, initially zeroed, with the
//! following structure:
//!
//! `... cccccccc ccHWCERT` (exact number of bits depends on the target word
//! size)
//!
//! Where the bit, if set, indicates:
//! * `T` - [`Sender`] half waker is stored
//! * `R` - [`Receiver`] half waker is stored
//! * `E` - error value of type `E` is stored
//! * `C` - [`Receiver`] half is closed
//! * `W` - [`Receiver`] half is closed, but there is a pending error
//! * `H` - one of the halves was dropped
//! * `c` - counter bits

use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::ptr::NonNull;
#[cfg(feature = "_atomics")]
use core::sync::atomic::AtomicUsize;
use core::task::Waker;

#[cfg(loom)]
use loom::sync::atomic::AtomicUsize;

pub use self::receiver::{Canceled, Receiver};
pub use self::sender::{Cancellation, SendError, Sender};
#[cfg(not(any(feature = "_atomics", loom)))]
use crate::sync::soft_atomic::Atomic;

mod receiver;
mod sender;

/// Creates a new pulse channel, returning the sender/receiver halves.
///
/// The [`Sender`] half is used to send a pack of pulses. The [`Receiver`] half
/// is a [`Stream`](futures::stream::Stream) that emits the number of pulses
/// generated since the last poll.
///
/// See [the module-level documentation](self) for details.
pub fn channel<E>() -> (Sender<E>, Receiver<E>) {
    let shared = unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(Shared::new()))) };
    let sender = Sender::new(shared);
    let receiver = Receiver::new(shared);
    (sender, receiver)
}

const TX_WAKER_STORED_SHIFT: u32 = 0;
const RX_WAKER_STORED_SHIFT: u32 = 1;
const ERR_STORED_SHIFT: u32 = 2;
const CLOSED_SHIFT: u32 = 3;
const CLOSED_WITH_ERR_SHIFT: u32 = 4;
const HALF_DROPPED_SHIFT: u32 = 5;
const PARAM_BITS: u32 = 6;

const TX_WAKER_STORED: usize = 1 << TX_WAKER_STORED_SHIFT;
const RX_WAKER_STORED: usize = 1 << RX_WAKER_STORED_SHIFT;
const ERR_STORED: usize = 1 << ERR_STORED_SHIFT;
const CLOSED: usize = 1 << CLOSED_SHIFT;
const CLOSED_WITH_ERR: usize = 1 << CLOSED_WITH_ERR_SHIFT;
const HALF_DROPPED: usize = 1 << HALF_DROPPED_SHIFT;

impl<T> Unpin for Sender<T> {}
impl<T> Unpin for Receiver<T> {}
unsafe impl<T: Send> Send for Sender<T> {}
unsafe impl<T: Send> Sync for Receiver<T> {}

struct Shared<E> {
    #[cfg(not(any(feature = "_atomics", loom)))]
    state: Atomic<usize>,
    #[cfg(any(feature = "_atomics", loom))]
    state: AtomicUsize,
    err: UnsafeCell<MaybeUninit<E>>,
    rx_waker: UnsafeCell<MaybeUninit<Waker>>,
    tx_waker: UnsafeCell<MaybeUninit<Waker>>,
}

impl<E> Shared<E> {
    fn new() -> Self {
        Self {
            #[cfg(not(any(feature = "_atomics", loom)))]
            state: Atomic::new(0),
            #[cfg(any(feature = "_atomics", loom))]
            state: AtomicUsize::new(0),
            err: UnsafeCell::new(MaybeUninit::uninit()),
            rx_waker: UnsafeCell::new(MaybeUninit::uninit()),
            tx_waker: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }
}
