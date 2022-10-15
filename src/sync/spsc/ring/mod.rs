//! A single-producer, single-consumer queue for sending values across
//! asynchronous tasks.
//!
//! This is a bounded channel that implements a ring buffer of values. Maximum
//! size of the ring buffer depends on the machine word size and defined by
//! [`MAX_CAPACITY`] constant.
//!
//! # Memory footprint
//!
//! Call to [`channel`] creates one allocation of an inner shared object. Each
//! returned half is a double-word-sized (wide) pointer to the shared object.
//!
//! The shared object consists of a the generic type `E`, array of generic types
//! `T` of length `capacity`, word-sized state field, and two double-word-sized
//! [`Waker`] objects.
//!
//! # State field structure
//!
//! Channel state is an atomic `usize` value, initially zeroed, with the
//! following structure:
//!
//! `llllllll ll... cccccccc ccHCERFT` (exact number of bits depends on the
//! target word size)
//!
//! Where the bit, if set, indicates:
//! * `T` - [`Sender`] half waker is stored for ready event
//! * `F` - [`Sender`] half waker is stored for flush event
//! * `R` - [`Receiver`] half waker is stored
//! * `E` - error value of type `E` is stored
//! * `C` - [`Receiver`] half is closed
//! * `H` - one of the halves was dropped
//! * `c` - ring buffer cursor value bits
//! * `l` - ring buffer length value bits
//!
//! The number of `c` bits equals to the number of `l` bits. If both `T` and `F`
//! set, the waker is stored for close event.

pub use self::receiver::{Receiver, TryNextError};
pub use self::sender::{SendError, Sender, TrySendError};
use alloc::alloc::{alloc, handle_alloc_error, Layout};
use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::ptr::{self, slice_from_raw_parts_mut, NonNull};
use core::task::Waker;

mod receiver;
mod sender;

/// Creates a bounded spsc channel for communicating between asynchronous tasks.
///
/// Being bounded, this channel provides backpressure to ensure that the sender
/// outpaces the receiver by only a limited amount. The channel's capacity is
/// set by the `capacity` argument.
///
/// The [`Receiver`] returned implements the [`Stream`](futures::stream::Stream)
/// trait, while [`Sender`] implements [`Sink`](futures::sink::Sink).
///
/// # Panics
///
/// If `capacity` exceeds [`MAX_CAPACITY`] constant or less than 2.
pub fn channel<T, E>(capacity: usize) -> (Sender<T, E>, Receiver<T, E>) {
    assert!(capacity > 1 && capacity <= MAX_CAPACITY);
    let shared = Shared::new(capacity);
    let sender = Sender::new(shared);
    let receiver = Receiver::new(shared);
    (sender, receiver)
}

/// Maximum capacity of the ring channel's inner ring buffer.
pub const MAX_CAPACITY: usize = 1 << COUNT_BITS;

const TX_READY_WAKER_STORED_SHIFT: u32 = 0;
const TX_FLUSH_WAKER_STORED_SHIFT: u32 = 1;
const RX_WAKER_STORED_SHIFT: u32 = 2;
const ERR_STORED_SHIFT: u32 = 3;
const CLOSED_SHIFT: u32 = 4;
const HALF_DROPPED_SHIFT: u32 = 5;
const PARAM_BITS: u32 = 6;
const COUNT_BITS: u32 = usize::BITS - PARAM_BITS >> 1;

const TX_READY_WAKER_STORED: usize = 1 << TX_READY_WAKER_STORED_SHIFT;
const TX_FLUSH_WAKER_STORED: usize = 1 << TX_FLUSH_WAKER_STORED_SHIFT;
const RX_WAKER_STORED: usize = 1 << RX_WAKER_STORED_SHIFT;
const ERR_STORED: usize = 1 << ERR_STORED_SHIFT;
const CLOSED: usize = 1 << CLOSED_SHIFT;
const HALF_DROPPED: usize = 1 << HALF_DROPPED_SHIFT;
const COUNT_MASK: usize = (1 << COUNT_BITS) - 1;

impl<T, E> Unpin for Sender<T, E> {}
impl<T, E> Unpin for Receiver<T, E> {}
unsafe impl<T: Send, E: Send> Send for Sender<T, E> {}
unsafe impl<T: Send, E: Send> Sync for Receiver<T, E> {}

#[cfg(all(feature = "atomics", not(loom)))]
type State = core::sync::atomic::AtomicUsize;
#[cfg(all(feature = "atomics", loom))]
type State = loom::sync::atomic::AtomicUsize;
#[cfg(not(feature = "atomics"))]
type State = crate::sync::soft_atomic::Atomic<usize>;

struct Header<E> {
    state: State,
    err: UnsafeCell<MaybeUninit<E>>,
    rx_waker: UnsafeCell<MaybeUninit<Waker>>,
    tx_waker: UnsafeCell<MaybeUninit<Waker>>,
}

#[repr(C)]
struct Shared<T, E> {
    hdr: Header<E>,
    buf: [UnsafeCell<MaybeUninit<T>>],
}

impl<T, E> Shared<T, E> {
    fn new(capacity: usize) -> NonNull<Self> {
        unsafe {
            let layout = Layout::new::<Header<E>>();
            let (layout, _) = layout.extend(Layout::array::<T>(capacity).unwrap()).unwrap();
            let layout = layout.pad_to_align();
            let ptr = NonNull::new(alloc(layout)).unwrap_or_else(|| handle_alloc_error(layout));
            let ptr = slice_from_raw_parts_mut(ptr.as_ptr(), capacity) as *mut Self;
            ptr::addr_of_mut!((*ptr).hdr.state).write(State::new(0));
            NonNull::new_unchecked(ptr)
        }
    }
}

fn has_waker(state: usize) -> bool {
    state & TX_READY_WAKER_STORED != 0 || state & TX_FLUSH_WAKER_STORED != 0
}

fn has_ready_waker(state: usize) -> bool {
    state & TX_READY_WAKER_STORED != 0 && state & TX_FLUSH_WAKER_STORED == 0
}

fn has_flush_waker(state: usize) -> bool {
    state & TX_READY_WAKER_STORED == 0 && state & TX_FLUSH_WAKER_STORED != 0
}

fn has_close_waker(state: usize) -> bool {
    state & TX_READY_WAKER_STORED != 0 && state & TX_FLUSH_WAKER_STORED != 0
}

fn set_ready_waker(state: usize) -> usize {
    state & !TX_FLUSH_WAKER_STORED | TX_READY_WAKER_STORED
}

fn set_flush_waker(state: usize) -> usize {
    state & !TX_READY_WAKER_STORED | TX_FLUSH_WAKER_STORED
}

fn set_close_waker(state: usize) -> usize {
    state | TX_READY_WAKER_STORED | TX_FLUSH_WAKER_STORED
}

fn get_length(state: usize) -> usize {
    state >> PARAM_BITS + COUNT_BITS
}

fn get_cursor(state: usize) -> usize {
    state >> PARAM_BITS & COUNT_MASK
}

fn add_cursor(mut cursor: usize, addition: usize, capacity: usize) -> usize {
    cursor += addition;
    if cursor >= capacity { cursor - capacity } else { cursor }
}

fn claim_next_unless_empty(state: usize, capacity: usize) -> usize {
    let length = get_length(state);
    if length > 0 { claim_next(state, capacity, length) } else { state }
}

fn claim_next_if_full(state: usize, capacity: usize) -> usize {
    let length = get_length(state);
    if state & CLOSED == 0 && length == capacity {
        claim_next(state, capacity, length)
    } else {
        state
    }
}

fn claim_next(state: usize, capacity: usize, length: usize) -> usize {
    state & (1 << PARAM_BITS) - 1
        | add_cursor(get_cursor(state), 1, capacity) << PARAM_BITS
        | length - 1 << PARAM_BITS + COUNT_BITS
}

fn add_length(state: usize, addition: usize) -> usize {
    if state & CLOSED == 0 { state + (addition << PARAM_BITS + COUNT_BITS) } else { state }
}
