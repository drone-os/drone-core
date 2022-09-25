//! A single-producer, single-consumer channel for counting events across
//! asynchronous tasks.
//!
//! It is similar to [`oneshot::channel<T>`], in the way how a single error
//! message of type `E` can be sent. And it is similar to [`ring::channel<T,
//! E>`] in the way that multiple values can be sent, but only of type `usize`.
//!
//! This channel can be seen as a shared counter. The sender half increments the
//! counter by a given value, while the receiver half clears the counter on each
//! poll and returns the number that was cleared. The size of the counter
//! depends on the machine word size and defined by [`CAPACITY`].
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
//! `... cccccccc cccHCERT` (exact number of bits depends on the target word
//! size)
//!
//! Where the bit, if set, indicates:
//! * `T` - [`Sender`] half waker is stored
//! * `R` - [`Receiver`] half waker is stored
//! * `E` - error value of type `E` is stored
//! * `C` - [`Receiver`] half is closed
//! * `H` - one of the halves was dropped
//! * `c` - counter value bits

use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::ptr::NonNull;
#[cfg(feature = "_atomics")]
use core::sync::atomic::AtomicUsize;
use core::task::Waker;

#[cfg(loom)]
use loom::sync::atomic::AtomicUsize;

pub use self::receiver::{Receiver, TryNextError};
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

/// Capacity of the pulse channel's inner counter.
pub const CAPACITY: usize = 1 << usize::BITS - PARAM_BITS;

const TX_WAKER_STORED_SHIFT: u32 = 0;
const RX_WAKER_STORED_SHIFT: u32 = 1;
const ERR_STORED_SHIFT: u32 = 2;
const CLOSED_SHIFT: u32 = 3;
const HALF_DROPPED_SHIFT: u32 = 4;
const PARAM_BITS: u32 = 5;

const TX_WAKER_STORED: usize = 1 << TX_WAKER_STORED_SHIFT;
const RX_WAKER_STORED: usize = 1 << RX_WAKER_STORED_SHIFT;
const ERR_STORED: usize = 1 << ERR_STORED_SHIFT;
const CLOSED: usize = 1 << CLOSED_SHIFT;
const HALF_DROPPED: usize = 1 << HALF_DROPPED_SHIFT;

impl<T> Unpin for Sender<T> {}
impl<T> Unpin for Receiver<T> {}
unsafe impl<T: Send> Send for Sender<T> {}
unsafe impl<T: Send> Sync for Receiver<T> {}

#[cfg(not(any(feature = "_atomics", loom)))]
type State = Atomic<usize>;
#[cfg(any(feature = "_atomics", loom))]
type State = AtomicUsize;

struct Shared<E> {
    state: State,
    err: UnsafeCell<MaybeUninit<E>>,
    rx_waker: UnsafeCell<MaybeUninit<Waker>>,
    tx_waker: UnsafeCell<MaybeUninit<Waker>>,
}

impl<E> Shared<E> {
    fn new() -> Self {
        Self {
            state: State::new(0),
            err: UnsafeCell::new(MaybeUninit::uninit()),
            rx_waker: UnsafeCell::new(MaybeUninit::uninit()),
            tx_waker: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }
}
