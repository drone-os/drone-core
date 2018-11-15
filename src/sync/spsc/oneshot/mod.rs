//! A single-producer, single-consumer oneshot channel.
//!
//! See [`oneshot::channel`] documentation for more.

mod receiver;
mod sender;

pub use self::receiver::{Receiver, RecvError};
pub use self::sender::Sender;

use alloc::sync::Arc;
use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicU8, Ordering};
use futures::task::Waker;
use sync::spsc::SpscInner;

const COMPLETE: u8 = 1 << 2;
const RX_LOCK: u8 = 1 << 1;
const TX_LOCK: u8 = 1;

struct Inner<T, E> {
  state: AtomicU8,
  data: UnsafeCell<Option<Result<T, E>>>,
  rx_waker: UnsafeCell<Option<Waker>>,
  tx_waker: UnsafeCell<Option<Waker>>,
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
      rx_waker: UnsafeCell::new(None),
      tx_waker: UnsafeCell::new(None),
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
    self.state.compare_exchange(current, new, success, failure)
  }

  #[inline(always)]
  unsafe fn rx_waker_mut(&self) -> &mut Option<Waker> {
    &mut *self.rx_waker.get()
  }

  #[inline(always)]
  unsafe fn tx_waker_mut(&self) -> &mut Option<Waker> {
    &mut *self.tx_waker.get()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use alloc::sync::Arc;
  use core::sync::atomic::{AtomicUsize, Ordering};
  use futures::prelude::*;

  thread_local! {
    static COUNTER: Arc<Counter> = Arc::new(Counter(AtomicUsize::new(0)));
  }

  struct Counter(AtomicUsize);

  impl task::Wake for Counter {
    fn wake(arc_self: &Arc<Self>) {
      arc_self.0.fetch_add(1, Ordering::Relaxed);
    }
  }

  #[test]
  fn send_sync() {
    let (mut rx, tx) = channel::<usize, ()>();
    assert_eq!(tx.send(Ok(314)), Ok(()));
    COUNTER.with(|counter| {
      let waker = task::Waker::from(Arc::clone(counter));
      let mut map = task::LocalMap::new();
      let mut cx = task::Context::without_spawn(&mut map, &waker);
      counter.0.store(0, Ordering::Relaxed);
      assert_eq!(rx.poll(&mut cx).unwrap(), Async::Ready(314));
      assert_eq!(counter.0.load(Ordering::Relaxed), 0);
    });
  }

  #[test]
  fn send_async() {
    let (mut rx, tx) = channel::<usize, ()>();
    COUNTER.with(|counter| {
      let waker = task::Waker::from(Arc::clone(counter));
      let mut map = task::LocalMap::new();
      let mut cx = task::Context::without_spawn(&mut map, &waker);
      counter.0.store(0, Ordering::Relaxed);
      assert_eq!(rx.poll(&mut cx).unwrap(), Async::Pending);
      assert_eq!(tx.send(Ok(314)), Ok(()));
      assert_eq!(rx.poll(&mut cx).unwrap(), Async::Ready(314));
      assert_eq!(counter.0.load(Ordering::Relaxed), 1);
    });
  }
}
