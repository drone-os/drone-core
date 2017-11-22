//! A single-producer, single-consumer channel based on a ring buffer.
//!
//! See [`ring::channel`] documentation for more details.
//!
//! [`ring::channel`]: fn.channel.html

mod receiver;
mod sender;

pub use self::receiver::Receiver;
pub use self::sender::{SendError, SendErrorKind, Sender};

use alloc::arc::Arc;
use alloc::raw_vec::RawVec;
use core::{cmp, mem, ptr, slice};
use core::cell::UnsafeCell;
use core::sync::atomic::{self, AtomicUsize};
use futures::task::Task;
use sync::spsc::SpscInner;

/// Maximum capacity of the channel.
pub const MAX_CAPACITY: usize = (1 << INDEX_BITS) - 1;

const INDEX_MASK: usize = (1 << INDEX_BITS) - 1;
const INDEX_BITS: usize = (mem::size_of::<usize>() * 8 - LOCK_BITS) / 2;
const LOCK_BITS: usize = 4;
const _RESERVED: usize = 1 << mem::size_of::<usize>() * 8 - 1;
const COMPLETE: usize = 1 << mem::size_of::<usize>() * 8 - 2;
const TX_LOCK: usize = 1 << mem::size_of::<usize>() * 8 - 3;
const RX_LOCK: usize = 1 << mem::size_of::<usize>() * 8 - 4;

// Layout of the state field:
//     LLLL_BBBB_CCCC
// Where L is lock bits, B is begin bits, and C is count bits.
//
// Begin range: [0; MAX_CAPACITY - 1]
// Count range: [0; MAX_CAPACITY]
struct Inner<T, E> {
  state: AtomicUsize,
  buffer: RawVec<T>,
  err: UnsafeCell<Option<E>>,
  tx_task: UnsafeCell<Option<Task>>,
  rx_task: UnsafeCell<Option<Task>>,
}

/// Creates a new asynchronous channel, returning the sender/receiver halves.
/// All data sent on the [`Sender`] will become available on the [`Receiver`] in
/// the same order as it was sent.
///
/// Only one ['Sender']/[`Receiver`] is supported.
///
/// [`Sender`]: struct.Sender.html
/// [`Receiver`]: struct.Receiver.html
#[inline]
pub fn channel<T, E>(capacity: usize) -> (Sender<T, E>, Receiver<T, E>) {
  let inner = Arc::new(Inner::new(capacity));
  let sender = Sender::new(Arc::clone(&inner));
  let receiver = Receiver::new(inner);
  (sender, receiver)
}

unsafe impl<T: Send, E: Send> Send for Inner<T, E> {}
unsafe impl<T: Send, E: Send> Sync for Inner<T, E> {}

impl<T, E> Inner<T, E> {
  #[inline(always)]
  fn new(capacity: usize) -> Self {
    assert!(capacity <= MAX_CAPACITY);
    Self {
      state: AtomicUsize::new(0),
      buffer: RawVec::with_capacity(capacity),
      err: UnsafeCell::new(None),
      tx_task: UnsafeCell::new(None),
      rx_task: UnsafeCell::new(None),
    }
  }
}

impl<T, E> Drop for Inner<T, E> {
  fn drop(&mut self) {
    let state = self.state_load(atomic::Ordering::Relaxed);
    let count = state & INDEX_MASK;
    let begin = state >> INDEX_BITS & INDEX_MASK;
    let end = begin.wrapping_add(count).wrapping_rem(self.buffer.cap());
    match begin.cmp(&end) {
      cmp::Ordering::Equal => unsafe {
        ptr::drop_in_place(slice::from_raw_parts_mut(
          self.buffer.ptr(),
          self.buffer.cap(),
        ));
      },
      cmp::Ordering::Less => unsafe {
        ptr::drop_in_place(slice::from_raw_parts_mut(
          self.buffer.ptr().offset(begin as isize),
          end - begin,
        ));
      },
      cmp::Ordering::Greater => unsafe {
        ptr::drop_in_place(slice::from_raw_parts_mut(self.buffer.ptr(), end));
        ptr::drop_in_place(slice::from_raw_parts_mut(
          self.buffer.ptr().offset(begin as isize),
          self.buffer.cap() - begin,
        ));
      },
    }
  }
}

impl<T, E> SpscInner<AtomicUsize, usize> for Inner<T, E> {
  const ZERO: usize = 0;
  const TX_LOCK: usize = TX_LOCK;
  const RX_LOCK: usize = RX_LOCK;
  const COMPLETE: usize = COMPLETE;

  #[inline(always)]
  fn state_load(&self, order: atomic::Ordering) -> usize {
    self.state.load(order)
  }

  #[inline(always)]
  fn state_exchange(
    &self,
    current: usize,
    new: usize,
    success: atomic::Ordering,
    failure: atomic::Ordering,
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
    let (mut tx, rx) = channel::<usize, ()>(10);
    assert_eq!(tx.send(314), Ok(()));
    drop(tx);
    let mut executor = executor::spawn(rx);
    COUNTER.with(|counter| {
      counter.0.store(0, Ordering::Relaxed);
      assert_eq!(
        executor.poll_stream_notify(counter, 0),
        Ok(Async::Ready(Some(314)))
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
    let (mut tx, rx) = channel::<usize, ()>(10);
    let mut executor = executor::spawn(rx);
    COUNTER.with(|counter| {
      counter.0.store(0, Ordering::Relaxed);
      assert_eq!(executor.poll_stream_notify(counter, 0), Ok(Async::NotReady));
      assert_eq!(tx.send(314), Ok(()));
      assert_eq!(
        executor.poll_stream_notify(counter, 0),
        Ok(Async::Ready(Some(314)))
      );
      assert_eq!(executor.poll_stream_notify(counter, 0), Ok(Async::NotReady));
      drop(tx);
      assert_eq!(
        executor.poll_stream_notify(counter, 0),
        Ok(Async::Ready(None))
      );
      assert_eq!(counter.0.load(Ordering::Relaxed), 2);
    });
  }
}
