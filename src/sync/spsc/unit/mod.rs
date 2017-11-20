//! A single-producer, single-consumer channel for `()`.
//!
//! See [`unit::channel`] documentation for more details.
//!
//! [`unit::channel`]: fn.channel.html

mod receiver;
mod sender;

pub use self::receiver::Receiver;
pub use self::sender::{SendError, Sender};

use alloc::arc::Arc;
use core::cell::UnsafeCell;
use core::mem::size_of;
use core::sync::atomic::{AtomicUsize, Ordering};
use futures::task::Task;
use sync::spsc::SpscInner;

/// Maximum capacity of the channel.
pub const MAX_CAPACITY: usize = 1 << size_of::<usize>() - LOCK_BITS;

const LOCK_MASK: usize = (1 << LOCK_BITS) - 1;
const LOCK_BITS: usize = 3;
const COMPLETE: usize = 1 << 2;
const TX_LOCK: usize = 1 << 1;
const RX_LOCK: usize = 1;

// Layout of the state field:
//     CCCC_LLLL
// Where C is counter bits, and L is lock bits.
struct Inner<E> {
  state: AtomicUsize,
  err: UnsafeCell<Option<E>>,
  tx_task: UnsafeCell<Option<Task>>,
  rx_task: UnsafeCell<Option<Task>>,
}

/// Creates a new asynchronous channel, returning the sender/receiver halves.
/// All units sent on the [`Sender`] will become available on the [`Receiver`].
///
/// Only one ['Sender']/[`Receiver`] is supported.
///
/// [`Sender`]: struct.Sender.html
/// [`Receiver`]: struct.Receiver.html
#[inline]
pub fn channel<E>() -> (Sender<E>, Receiver<E>) {
  let inner = Arc::new(Inner::new());
  let sender = Sender::new(Arc::clone(&inner));
  let receiver = Receiver::new(inner);
  (sender, receiver)
}

unsafe impl<E: Send> Send for Inner<E> {}
unsafe impl<E: Send> Sync for Inner<E> {}

impl<E> Inner<E> {
  #[inline(always)]
  fn new() -> Self {
    Self {
      state: AtomicUsize::new(0),
      err: UnsafeCell::new(None),
      tx_task: UnsafeCell::new(None),
      rx_task: UnsafeCell::new(None),
    }
  }
}

impl<E> SpscInner<AtomicUsize, usize> for Inner<E> {
  const ZERO: usize = 0;
  const TX_LOCK: usize = TX_LOCK;
  const RX_LOCK: usize = RX_LOCK;
  const COMPLETE: usize = COMPLETE;

  #[inline(always)]
  fn state_load(&self, order: Ordering) -> usize {
    self.state.load(order)
  }

  #[inline(always)]
  fn state_exchange(
    &self,
    current: usize,
    new: usize,
    success: Ordering,
    failure: Ordering,
  ) -> Result<usize, usize> {
    self.state.compare_exchange(current, new, success, failure)
  }

  #[inline(always)]
  unsafe fn tx_task_mut(&self) -> &mut Option<Task> {
    &mut *self.tx_task.get()
  }

  #[inline(always)]
  unsafe fn rx_task_mut(&self) -> &mut Option<Task> {
    &mut *self.rx_task.get()
  }
}
