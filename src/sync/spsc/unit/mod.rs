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
use futures::task::Waker;
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
  rx_waker: UnsafeCell<Option<Waker>>,
  tx_waker: UnsafeCell<Option<Waker>>,
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
      rx_waker: UnsafeCell::new(None),
      tx_waker: UnsafeCell::new(None),
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
  use alloc::arc::Arc;
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
    let (mut rx, mut tx) = channel::<()>();
    assert_eq!(tx.send().unwrap(), ());
    drop(tx);
    COUNTER.with(|counter| {
      let waker = task::Waker::from(Arc::clone(counter));
      let mut map = task::LocalMap::new();
      let mut cx = task::Context::without_spawn(&mut map, &waker);
      counter.0.store(0, Ordering::Relaxed);
      assert_eq!(
        rx.poll_next(&mut cx),
        Ok(Async::Ready(Some(())))
      );
      assert_eq!(rx.poll_next(&mut cx), Ok(Async::Ready(None)));
      assert_eq!(counter.0.load(Ordering::Relaxed), 0);
    });
  }

  #[test]
  fn send_async() {
    let (mut rx, mut tx) = channel::<()>();
    COUNTER.with(|counter| {
      let waker = task::Waker::from(Arc::clone(counter));
      let mut map = task::LocalMap::new();
      let mut cx = task::Context::without_spawn(&mut map, &waker);
      counter.0.store(0, Ordering::Relaxed);
      assert_eq!(rx.poll_next(&mut cx), Ok(Async::Pending));
      assert_eq!(tx.send().unwrap(), ());
      assert_eq!(
        rx.poll_next(&mut cx),
        Ok(Async::Ready(Some(())))
      );
      assert_eq!(rx.poll_next(&mut cx), Ok(Async::Pending));
      drop(tx);
      assert_eq!(rx.poll_next(&mut cx), Ok(Async::Ready(None)));
      assert_eq!(counter.0.load(Ordering::Relaxed), 2);
    });
  }
}
