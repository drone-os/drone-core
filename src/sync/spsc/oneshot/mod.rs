//! A single-producer, single-consumer oneshot channel.
//!
//! See [`oneshot::channel`] documentation for more details.
//!
//! [`oneshot::channel`]: fn.channel.html

mod receiver;
mod sender;

pub use self::receiver::*;
pub use self::sender::*;

use alloc::arc::Arc;
use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicU8, Ordering};
use futures::task::Task;
use sync::spsc::SpscInner;

const COMPLETE: u8 = 1 << 2;
const TX_LOCK: u8 = 1 << 1;
const RX_LOCK: u8 = 1;

struct Inner<R, E> {
  state: AtomicU8,
  data: UnsafeCell<Option<Result<R, E>>>,
  tx_task: UnsafeCell<Option<Task>>,
  rx_task: UnsafeCell<Option<Task>>,
}

/// Creates a new asynchronous channel, returning the sender/receiver halves.
/// The data sent on the [`Sender`] will become available on the [`Receiver`].
///
/// Only one ['Sender']/[`Receiver`] is supported.
///
/// [`Sender`]: struct.Sender.html
/// [`Receiver`]: struct.Receiver.html
#[inline]
pub fn channel<R, E>() -> (Sender<R, E>, Receiver<R, E>) {
  let inner = Arc::new(Inner::new());
  let sender = Sender::new(Arc::clone(&inner));
  let receiver = Receiver::new(inner);
  (sender, receiver)
}

unsafe impl<R: Send, E: Send> Send for Inner<R, E> {}
unsafe impl<R: Send, E: Send> Sync for Inner<R, E> {}

impl<R, E> SpscInner<AtomicU8, u8> for Inner<R, E> {
  const ZERO: u8 = 0;
  const TX_LOCK: u8 = TX_LOCK;
  const RX_LOCK: u8 = RX_LOCK;
  const COMPLETE: u8 = COMPLETE;

  #[inline(always)]
  fn new() -> Self {
    Self {
      state: AtomicU8::new(0),
      data: UnsafeCell::new(None),
      tx_task: UnsafeCell::new(None),
      rx_task: UnsafeCell::new(None),
    }
  }

  #[inline(always)]
  fn state_load(&self, order: Ordering) -> u8 {
    self.state.load(order)
  }

  #[inline(always)]
  fn state_exchange(
    &self,
    current: u8,
    new: u8,
    success: Ordering,
    failure: Ordering,
  ) -> Result<u8, u8> {
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
