//! A single-producer, single-consumer channel for `()`.
//!
//! See [`unit::channel`](channel) documentation for more details.

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
pub const MAX_CAPACITY: usize = 1 << size_of::<usize>() * 8 - LOCK_BITS;

const LOCK_MASK: usize = (1 << LOCK_BITS) - 1;
const LOCK_BITS: usize = 3;
const COMPLETE: usize = 1 << 2;
const RX_LOCK: usize = 1 << 1;
const TX_LOCK: usize = 1;

// Layout of the state field:
//     CCCC_LLLL
// Where C is counter bits, and L is lock bits.
struct Inner<E> {
  state: AtomicUsize,
  err: UnsafeCell<Option<E>>,
  rx_task: UnsafeCell<Option<Task>>,
  tx_task: UnsafeCell<Option<Task>>,
}

/// Creates a new asynchronous channel, returning the receiver/sender halves.
/// All units sent on the [`Sender`] will become available on the [`Receiver`].
///
/// Only one [`Receiver`]/[`Sender`] is supported.
///
/// [`Receiver`]: Receiver
/// [`Sender`]: Sender
#[inline]
pub fn channel<E>() -> (Receiver<E>, Sender<E>) {
  let inner = Arc::new(Inner::new());
  let receiver = Receiver::new(Arc::clone(&inner));
  let sender = Sender::new(inner);
  (receiver, sender)
}

unsafe impl<E: Send> Send for Inner<E> {}
unsafe impl<E: Send> Sync for Inner<E> {}

impl<E> Inner<E> {
  #[inline(always)]
  fn new() -> Self {
    Self {
      state: AtomicUsize::new(0),
      err: UnsafeCell::new(None),
      rx_task: UnsafeCell::new(None),
      tx_task: UnsafeCell::new(None),
    }
  }
}

impl<E> SpscInner<AtomicUsize, usize> for Inner<E> {
  const ZERO: usize = 0;
  const RX_LOCK: usize = RX_LOCK;
  const TX_LOCK: usize = TX_LOCK;
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
    self
      .state
      .compare_exchange(current, new, success, failure)
  }

  #[inline(always)]
  unsafe fn rx_task_mut(&self) -> &mut Option<Task> {
    &mut *self.rx_task.get()
  }

  #[inline(always)]
  unsafe fn tx_task_mut(&self) -> &mut Option<Task> {
    &mut *self.tx_task.get()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use alloc::arc::Arc;
  use core::sync::atomic::{AtomicUsize, Ordering};
  use futures::executor::{self, Notify};

  thread_local! {
    static COUNTER: Arc<Counter> = Arc::new(Counter(AtomicUsize::new(0)));
  }

  struct Counter(AtomicUsize);

  impl Notify for Counter {
    fn notify(&self, _id: usize) {
      self.0.fetch_add(1, Ordering::Relaxed);
    }
  }

  #[test]
  fn send_sync() {
    let (rx, mut tx) = channel::<()>();
    assert_eq!(tx.send().unwrap(), ());
    drop(tx);
    let mut executor = executor::spawn(rx);
    COUNTER.with(|counter| {
      counter.0.store(0, Ordering::Relaxed);
      assert_eq!(
        executor.poll_stream_notify(counter, 0),
        Ok(Async::Ready(Some(())))
      );
      assert_eq!(
        executor.poll_stream_notify(counter, 0),
        Ok(Async::Ready(None))
      );
      assert_eq!(counter.0.load(Ordering::Relaxed), 0);
    });
  }

  #[test]
  fn send_async() {
    let (rx, mut tx) = channel::<()>();
    let mut executor = executor::spawn(rx);
    COUNTER.with(|counter| {
      counter.0.store(0, Ordering::Relaxed);
      assert_eq!(
        executor.poll_stream_notify(counter, 0),
        Ok(Async::NotReady)
      );
      assert_eq!(tx.send().unwrap(), ());
      assert_eq!(
        executor.poll_stream_notify(counter, 0),
        Ok(Async::Ready(Some(())))
      );
      assert_eq!(
        executor.poll_stream_notify(counter, 0),
        Ok(Async::NotReady)
      );
      drop(tx);
      assert_eq!(
        executor.poll_stream_notify(counter, 0),
        Ok(Async::Ready(None))
      );
      assert_eq!(counter.0.load(Ordering::Relaxed), 2);
    });
  }
}
