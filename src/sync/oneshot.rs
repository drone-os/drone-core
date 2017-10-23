//! A one-shot, futures-aware channel.

use alloc::arc::Arc;
use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicU8, Ordering};
use core::sync::atomic::Ordering::*;
use futures::{Async, Future, Poll};
use futures::task::{self, Task};

/// Represents the completion half of a oneshot through which the result of a
/// computation is signaled.
///
/// This is created by the [`oneshot::channel`] function.
///
/// [`oneshot::channel`]: fn.channel.html
pub struct Sender<T> {
  inner: Arc<Inner<T>>,
}

/// A future representing the completion of a computation happening elsewhere in
/// memory.
///
/// This is created by the [`oneshot::channel`] function.
///
/// [`oneshot::channel`]: fn.channel.html
#[must_use]
pub struct Receiver<T> {
  inner: Arc<Inner<T>>,
}

/// Error returned from a [`Receiver]` whenever the corresponding [`Sender`] is
/// dropped.
///
/// [`Sender`]: struct.Sender.html
/// [`Receiver`]: struct.Receiver.html
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Canceled;

struct Inner<T> {
  lock: AtomicU8,
  data: UnsafeCell<Option<T>>,
  tx_task: UnsafeCell<Option<Task>>,
  rx_task: UnsafeCell<Option<Task>>,
}

bitflags! {
  struct Lock: u8 {
    const COMPLETE = 0b0100;
    const TX_LOCK = 0b0010;
    const RX_LOCK = 0b0001;
  }
}

/// Creates a new futures-aware, one-shot channel.
///
/// This function is similar to Rust's channels found in the standard library.
/// Two halves are returned, the first of which is a [`Sender`] handle, used to
/// signal the end of a computation and provide its value. The second half is a
/// [`Receiver`] which implements the `Future` trait, resolving to the value
/// that was given to the [`Sender`] handle.
///
/// Each half can be separately owned and sent across threads/tasks.
///
/// [`Sender`]: struct.Sender.html
/// [`Receiver`]: struct.Receiver.html
pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
  let inner = Arc::new(Inner::new());
  let sender = Sender::new(Arc::clone(&inner));
  let receiver = Receiver::new(inner);
  (sender, receiver)
}

impl<T> Sender<T> {
  fn new(inner: Arc<Inner<T>>) -> Self {
    Self { inner }
  }

  /// Completes this oneshot with a successful result.
  ///
  /// This function will consume `self` and indicate to the other end, the
  /// [`Receiver`], that the value provided is the result of the computation
  /// this represents.
  ///
  /// If the value is successfully enqueued for the remote end to receive,
  /// then `Ok(())` is returned. If the receiving end was deallocated before
  /// this function was called, however, then `Err` is returned with the value
  /// provided.
  ///
  /// [`Receiver`]: struct.Receiver.html
  pub fn send(self, t: T) -> Result<(), T> {
    self.inner.send(t)
  }

  /// Polls this [`Sender`] half to detect whether the [`Receiver`] this has
  /// paired with has gone away.
  ///
  /// This function can be used to learn about when the [`Receiver`] (consumer)
  /// half has gone away and nothing will be able to receive a message sent from
  /// [`send`].
  ///
  /// If `Ready` is returned then it means that the [`Receiver`] has disappeared
  /// and the result this [`Sender`] would otherwise produce should no longer be
  /// produced.
  ///
  /// If `NotReady` is returned then the [`Receiver`] is still alive and may be
  /// able to receive a message if sent. The current task, however, is scheduled
  /// to receive a notification if the corresponding [`Receiver`] goes away.
  ///
  /// # Panics
  ///
  /// Like `Future::poll`, this function will panic if it's not called from
  /// within the context of a task. In other words, this should only ever be
  /// called from inside another future.
  ///
  /// If you're calling this function from a context that does not have a task,
  /// then you can use the [`is_canceled`] API instead.
  ///
  /// [`Sender`]: struct.Sender.html
  /// [`Receiver`]: struct.Receiver.html
  /// [`send`]: struct.Receiver.html#method.send
  /// [`is_canceled`]: struct.Receiver.html#method.is_canceled
  pub fn poll_cancel(&mut self) -> Poll<(), ()> {
    self.inner.poll_cancel()
  }

  /// Tests to see whether this [`Sender`]'s corresponding [`Receiver`] has gone
  /// away.
  ///
  /// This function can be used to learn about when the [`Receiver`] (consumer)
  /// half has gone away and nothing will be able to receive a message sent from
  /// [`send`].
  ///
  /// Note that this function is intended to *not* be used in the context of a
  /// future. If you're implementing a future you probably want to call the
  /// [`poll_cancel`] function which will block the current task if the
  /// cancellation hasn't happened yet. This can be useful when working on a
  /// non-futures related thread, though, which would otherwise panic if
  /// [`poll_cancel`] were called.
  ///
  /// [`Sender`]: struct.Sender.html
  /// [`Receiver`]: struct.Receiver.html
  /// [`send`]: struct.Receiver.html#method.send
  /// [`poll_cancel`]: struct.Receiver.html#method.poll_cancel
  pub fn is_canceled(&self) -> bool {
    self.inner.is_canceled()
  }
}

impl<T> Drop for Sender<T> {
  fn drop(&mut self) {
    self.inner.drop_tx();
  }
}

impl<T> Receiver<T> {
  fn new(inner: Arc<Inner<T>>) -> Self {
    Self { inner }
  }

  /// Gracefully close this receiver, preventing sending any future messages.
  ///
  /// Any [`send`] operation which happens after this method returns is
  /// guaranteed to fail. Once this method is called the normal [`poll`] method
  /// can be used to determine whether a message was actually sent or not. If
  /// [`Canceled`] is returned from [`poll`] then no message was sent.
  ///
  /// [`Canceled`]: struct.Canceled.html
  /// [`send`]: struct.Receiver.html#method.send
  /// [`poll`]: struct.Receiver.html#method.poll
  pub fn close(&mut self) {
    self.inner.close_rx()
  }
}

impl<T> Future for Receiver<T> {
  type Item = T;
  type Error = Canceled;

  fn poll(&mut self) -> Poll<T, Canceled> {
    self.inner.recv()
  }
}

impl<T> Drop for Receiver<T> {
  fn drop(&mut self) {
    self.inner.drop_rx();
  }
}

unsafe impl<T: Send> Send for Inner<T> {}
unsafe impl<T: Send> Sync for Inner<T> {}

// Sender half.
impl<T> Inner<T> {
  fn send(&self, t: T) -> Result<(), T> {
    if self.is_canceled() {
      Err(t)
    } else {
      unsafe { *self.data.get() = Some(t) };
      Ok(())
    }
  }

  fn poll_cancel(&self) -> Poll<(), ()> {
    self
      .try_lock_task(Lock::TX_LOCK)
      .and_then(|_| {
        self.set_task(Lock::TX_LOCK, unsafe { &mut *self.tx_task.get() })
      })
      .unwrap_or_else(|| Ok(Async::Ready(())))
  }

  fn drop_tx(&self) {
    self
      .update(Acquire, |lock| {
        let locked = !lock.intersects(Lock::RX_LOCK);
        lock.insert(Lock::RX_LOCK);
        lock.insert(Lock::COMPLETE);
        Some(locked)
      })
      .map(|locked| if locked {
        unsafe { (*self.rx_task.get()).take().map(|task| task.notify()) };
        self.update(Release, |lock| {
          lock.toggle(Lock::RX_LOCK);
          Some(())
        });
      });
  }

  fn is_canceled(&self) -> bool {
    Lock::from_bits_truncate(self.lock.load(Relaxed)).intersects(Lock::COMPLETE)
  }
}

// Receiver half.
impl<T> Inner<T> {
  fn recv(&self) -> Poll<T, Canceled> {
    self
      .try_lock_task(Lock::RX_LOCK)
      .and_then(|_| {
        self.set_task(Lock::RX_LOCK, unsafe { &mut *self.rx_task.get() })
      })
      .unwrap_or_else(|| {
        let data = unsafe { &mut *self.data.get() };
        data.take().ok_or(Canceled).map(Async::Ready)
      })
  }

  fn close_rx(&self) {
    self
      .update(Acquire, |lock| {
        let locked = !lock.intersects(Lock::TX_LOCK);
        lock.insert(Lock::TX_LOCK);
        lock.insert(Lock::COMPLETE);
        Some(locked)
      })
      .map(|locked| if locked {
        unsafe { (*self.tx_task.get()).take().map(|task| task.notify()) };
        self.update(Release, |lock| {
          lock.toggle(Lock::TX_LOCK);
          Some(())
        });
      });
  }

  fn drop_rx(&self) {
    self
      .update(Acquire, |lock| {
        let mut mask = Lock::empty();
        if !lock.intersects(Lock::TX_LOCK) {
          mask.insert(Lock::TX_LOCK);
        }
        if !lock.intersects(Lock::RX_LOCK) {
          mask.insert(Lock::RX_LOCK);
        }
        lock.insert(mask);
        lock.insert(Lock::COMPLETE);
        Some(mask)
      })
      .map(|mask| {
        unsafe {
          if mask.intersects(Lock::RX_LOCK) {
            (*self.rx_task.get()).take();
          }
          if mask.intersects(Lock::TX_LOCK) {
            (*self.tx_task.get()).take().map(|task| task.notify());
          }
        }
        if !mask.is_empty() {
          self.update(Release, |lock| {
            lock.toggle(mask);
            Some(())
          });
        }
      });
  }
}

// Shared methods.
impl<T> Inner<T> {
  fn new() -> Self {
    Self {
      lock: AtomicU8::new(0),
      data: UnsafeCell::new(None),
      tx_task: UnsafeCell::new(None),
      rx_task: UnsafeCell::new(None),
    }
  }

  fn try_lock_task(&self, flag: Lock) -> Option<()> {
    self.update(Acquire, |lock| {
      if lock.intersects(Lock::COMPLETE) || lock.intersects(flag) {
        None
      } else {
        lock.insert(flag);
        Some(())
      }
    })
  }

  fn set_task<R, E>(
    &self,
    flag: Lock,
    task: &mut Option<Task>,
  ) -> Option<Poll<R, E>> {
    *task = Some(task::current());
    self.update(Release, |lock| {
      lock.toggle(flag);
      Some(())
    });
    if self.is_canceled() {
      None
    } else {
      Some(Ok(Async::NotReady))
    }
  }

  fn update<F, R>(&self, ordering: Ordering, f: F) -> Option<R>
  where
    F: Fn(&mut Lock) -> Option<R>,
  {
    loop {
      let old = self.lock.load(Relaxed);
      let mut new = Lock::from_bits_truncate(old);
      let result = f(&mut new);
      if result.is_none() {
        break result;
      }
      if self.lock.compare_and_swap(old, new.bits(), ordering) == old {
        break result;
      }
    }
  }
}
