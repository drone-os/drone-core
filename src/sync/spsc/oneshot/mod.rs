//! A single-producer, single-consumer oneshot channel.
//!
//! See [`oneshot::channel`] documentation for more.

mod receiver;
mod sender;

pub use self::{
  receiver::{Receiver, RecvError},
  sender::Sender,
};

use crate::sync::spsc::SpscInner;
use alloc::sync::Arc;
use core::{
  cell::UnsafeCell,
  sync::atomic::{AtomicU8, Ordering},
  task::Waker,
};

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
  #[inline]
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

  #[inline]
  fn state_load(&self, order: Ordering) -> u8 {
    self.state.load(order)
  }

  #[inline]
  fn state_exchange(
    &self,
    current: u8,
    new: u8,
    success: Ordering,
    failure: Ordering,
  ) -> Result<u8, u8> {
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
    future::Future,
    pin::Pin,
    sync::atomic::{AtomicUsize, Ordering::*},
    task::{Poll, RawWaker, RawWakerVTable, Waker},
  };

  struct Counter(AtomicUsize);

  impl Counter {
    fn to_waker(&'static self) -> Waker {
      unsafe fn clone(counter: *const ()) -> RawWaker {
        RawWaker::new(counter, &VTABLE)
      }
      unsafe fn wake(counter: *const ()) {
        (*(counter as *const Counter)).0.fetch_add(1, Relaxed);
      }
      static VTABLE: RawWakerVTable = RawWakerVTable { clone, wake, drop };
      unsafe {
        Waker::new_unchecked(RawWaker::new(
          self as *const _ as *const (),
          &VTABLE,
        ))
      }
    }
  }

  #[test]
  fn send_sync() {
    static COUNTER: Counter = Counter(AtomicUsize::new(0));
    let (mut rx, tx) = channel::<usize, ()>();
    assert_eq!(tx.send(Ok(314)), Ok(()));
    let waker = COUNTER.to_waker();
    COUNTER.0.store(0, Ordering::Relaxed);
    assert_eq!(Pin::new(&mut rx).poll(&waker), Poll::Ready(Ok(314)));
    assert_eq!(COUNTER.0.load(Ordering::Relaxed), 0);
  }

  #[test]
  fn send_async() {
    static COUNTER: Counter = Counter(AtomicUsize::new(0));
    let (mut rx, tx) = channel::<usize, ()>();
    let waker = COUNTER.to_waker();
    COUNTER.0.store(0, Ordering::Relaxed);
    assert_eq!(Pin::new(&mut rx).poll(&waker), Poll::Pending);
    assert_eq!(tx.send(Ok(314)), Ok(()));
    assert_eq!(Pin::new(&mut rx).poll(&waker), Poll::Ready(Ok(314)));
    assert_eq!(COUNTER.0.load(Ordering::Relaxed), 1);
  }
}
