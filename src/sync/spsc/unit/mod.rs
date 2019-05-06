//! A single-producer, single-consumer channel for `()`.
//!
//! See [`unit::channel`] documentation for more details.

mod receiver;
mod sender;

pub use self::{
  receiver::Receiver,
  sender::{SendError, Sender},
};

use crate::sync::spsc::SpscInner;
use alloc::sync::Arc;
use core::{
  cell::UnsafeCell,
  mem::size_of,
  sync::atomic::{AtomicUsize, Ordering},
  task::Waker,
};

/// Maximum capacity of the channel.
pub const MAX_CAPACITY: usize = 1 << size_of::<usize>() * 8 - LOCK_BITS;

const LOCK_MASK: usize = (1 << LOCK_BITS) - 1;
const LOCK_BITS: usize = 3;
const COMPLETE: usize = 1 << 2;
const RX_LOCK: usize = 1 << 1;
const TX_LOCK: usize = 1;

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
  #[inline]
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

  #[inline]
  fn state_load(&self, order: Ordering) -> usize {
    self.state.load(order)
  }

  #[inline]
  fn state_exchange(
    &self,
    current: usize,
    new: usize,
    success: Ordering,
    failure: Ordering,
  ) -> Result<usize, usize> {
    self.state.compare_exchange(current, new, success, failure)
  }

  #[inline]
  unsafe fn rx_waker_mut(&self) -> &mut Option<Waker> {
    &mut *self.rx_waker.get()
  }

  #[inline]
  unsafe fn tx_waker_mut(&self) -> &mut Option<Waker> {
    &mut *self.tx_waker.get()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use core::{
    num::NonZeroUsize,
    pin::Pin,
    sync::atomic::{AtomicUsize, Ordering::*},
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
  };
  use futures::stream::Stream;

  struct Counter(AtomicUsize);

  impl Counter {
    fn to_waker(&'static self) -> Waker {
      unsafe fn clone(counter: *const ()) -> RawWaker {
        RawWaker::new(counter, &VTABLE)
      }
      unsafe fn wake(counter: *const ()) {
        (*(counter as *const Counter)).0.fetch_add(1, Relaxed);
      }
      static VTABLE: RawWakerVTable =
        RawWakerVTable::new(clone, wake, wake, drop);
      unsafe {
        Waker::from_raw(RawWaker::new(self as *const _ as *const (), &VTABLE))
      }
    }
  }

  #[test]
  fn send_sync() {
    static COUNTER: Counter = Counter(AtomicUsize::new(0));
    let (mut rx, mut tx) = channel::<()>();
    assert_eq!(tx.send().unwrap(), ());
    drop(tx);
    let waker = COUNTER.to_waker();
    let mut cx = Context::from_waker(&waker);
    assert_eq!(
      Pin::new(&mut rx).poll_next(&mut cx),
      Poll::Ready(Some(Ok(())))
    );
    assert_eq!(Pin::new(&mut rx).poll_next(&mut cx), Poll::Ready(None));
    assert_eq!(COUNTER.0.load(Ordering::Relaxed), 0);
  }

  #[test]
  fn send_async() {
    static COUNTER: Counter = Counter(AtomicUsize::new(0));
    let (mut rx, mut tx) = channel::<()>();
    let waker = COUNTER.to_waker();
    let mut cx = Context::from_waker(&waker);
    assert_eq!(Pin::new(&mut rx).poll_next(&mut cx), Poll::Pending);
    assert_eq!(tx.send().unwrap(), ());
    assert_eq!(
      Pin::new(&mut rx).poll_next(&mut cx),
      Poll::Ready(Some(Ok(())))
    );
    assert_eq!(Pin::new(&mut rx).poll_next(&mut cx), Poll::Pending);
    drop(tx);
    assert_eq!(Pin::new(&mut rx).poll_next(&mut cx), Poll::Ready(None));
    assert_eq!(COUNTER.0.load(Ordering::Relaxed), 2);
  }

  #[test]
  fn send_err() {
    static COUNTER: Counter = Counter(AtomicUsize::new(0));
    let (mut rx, tx) = channel::<()>();
    assert_eq!(tx.send_err(()).unwrap(), ());
    let waker = COUNTER.to_waker();
    let mut cx = Context::from_waker(&waker);
    assert_eq!(
      Pin::new(&mut rx).poll_next(&mut cx),
      Poll::Ready(Some(Err(())))
    );
    assert_eq!(Pin::new(&mut rx).poll_next(&mut cx), Poll::Ready(None));
    assert_eq!(COUNTER.0.load(Ordering::Relaxed), 0);
  }

  #[test]
  fn recv_all() {
    static COUNTER: Counter = Counter(AtomicUsize::new(0));
    let (mut rx, mut tx) = channel::<()>();
    let waker = COUNTER.to_waker();
    let mut cx = Context::from_waker(&waker);
    assert_eq!(Pin::new(&mut rx).poll_all(&mut cx), Poll::Pending);
    assert_eq!(tx.send().unwrap(), ());
    assert_eq!(tx.send().unwrap(), ());
    assert_eq!(tx.send().unwrap(), ());
    assert_eq!(
      Pin::new(&mut rx).poll_all(&mut cx),
      Poll::Ready(Some(Ok(NonZeroUsize::new(3).unwrap())))
    );
    assert_eq!(Pin::new(&mut rx).poll_all(&mut cx), Poll::Pending);
    drop(tx);
    assert_eq!(Pin::new(&mut rx).poll_all(&mut cx), Poll::Ready(None));
    assert_eq!(COUNTER.0.load(Ordering::Relaxed), 4);
  }
}
