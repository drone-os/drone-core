//! A single-producer, single-consumer oneshot channel.
//!
//! See [`oneshot::channel`](channel) documentation for more.

mod receiver;
mod sender;

pub use self::receiver::{Receiver, RecvError};
pub use self::sender::Sender;

use alloc::arc::Arc;
use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicU8, Ordering};
use futures::task::Task;
use sync::spsc::SpscInner;

const COMPLETE: u8 = 1 << 2;
const RX_LOCK: u8 = 1 << 1;
const TX_LOCK: u8 = 1;

struct Inner<T, E> {
  state: AtomicU8,
  data: UnsafeCell<Option<Result<T, E>>>,
  rx_task: UnsafeCell<Option<Task>>,
  tx_task: UnsafeCell<Option<Task>>,
}

/// Creates a new asynchronous channel, returning the receiver/sender halves.
/// The data sent on the [`Sender`] will become available on the [`Receiver`].
///
/// Only one [`Receiver`]/[`Sender`] is supported.
///
/// [`Receiver`]: Receiver
/// [`Sender`]: Sender
#[inline]
pub fn channel<T, E>() -> (Receiver<T, E>, Sender<T, E>) {
  let inner = Arc::new(Inner::new());
  let receiver = Receiver::new(Arc::clone(&inner));
  let sender = Sender::new(inner);
  (receiver, sender)
}

unsafe impl<T: Send, E: Send> Send for Inner<T, E> {}
unsafe impl<T: Send, E: Send> Sync for Inner<T, E> {}

impl<T, E> Inner<T, E> {
  #[inline(always)]
  fn new() -> Self {
    Self {
      state: AtomicU8::new(0),
      data: UnsafeCell::new(None),
      rx_task: UnsafeCell::new(None),
      tx_task: UnsafeCell::new(None),
    }
  }
}

impl<T, E> SpscInner<AtomicU8, u8> for Inner<T, E> {
  const ZERO: u8 = 0;
  const RX_LOCK: u8 = RX_LOCK;
  const TX_LOCK: u8 = TX_LOCK;
  const COMPLETE: u8 = COMPLETE;

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
    let (rx, tx) = channel::<usize, ()>();
    assert_eq!(tx.send(Ok(314)), Ok(()));
    let mut executor = executor::spawn(rx);
    COUNTER.with(|counter| {
      counter.0.store(0, Ordering::Relaxed);
      assert_eq!(
        executor.poll_future_notify(counter, 0).unwrap(),
        Async::Ready(314)
      );
      assert_eq!(counter.0.load(Ordering::Relaxed), 0);
    });
  }

  #[test]
  fn send_async() {
    let (rx, tx) = channel::<usize, ()>();
    let mut executor = executor::spawn(rx);
    COUNTER.with(|counter| {
      counter.0.store(0, Ordering::Relaxed);
      assert_eq!(
        executor.poll_future_notify(counter, 0).unwrap(),
        Async::NotReady
      );
      assert_eq!(tx.send(Ok(314)), Ok(()));
      assert_eq!(
        executor.poll_future_notify(counter, 0).unwrap(),
        Async::Ready(314)
      );
      assert_eq!(counter.0.load(Ordering::Relaxed), 1);
    });
  }
}
