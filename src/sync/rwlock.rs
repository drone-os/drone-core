//! A reader-writer lock.
//!
//! See [`RwLock`] for more details.
//!
//! [`RwLock`]: struct.RwLock.html

use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering::*;

const WRITE_LOCK: usize = usize::max_value();
const NO_LOCK: usize = usize::min_value();

/// A reader-writer lock.
///
/// This lock supports only [`try_read`] and [`try_write`] method, and hence
/// never blocks.
///
/// [`try_read`]: #method.try_read
/// [`try_write`]: #method.try_write
pub struct RwLock<T> {
  lock: AtomicUsize,
  data: UnsafeCell<T>,
}

/// RAII structure used to release the shared read access of a lock when
/// dropped.
///
/// This structure is created by the [`try_read`] method on [`RwLock`].
///
/// [`RwLock`]: struct.RwLock.html
/// [`try_read`]: struct.RwLock.html#method.try_read
#[must_use]
pub struct RwLockReadGuard<'a, T>
where
  T: 'a,
{
  lock: &'a RwLock<T>,
}

/// RAII structure used to release the exclusive write access of a lock when
/// dropped.
///
/// This structure is created by the [`try_write`] method on [`RwLock`].
///
/// [`RwLock`]: struct.RwLock.html
/// [`try_write`]: struct.RwLock.html#method.try_write
#[must_use]
pub struct RwLockWriteGuard<'a, T>
where
  T: 'a,
{
  lock: &'a RwLock<T>,
}

unsafe impl<T: Send + Sync> Send for RwLock<T> {}
unsafe impl<T: Send + Sync> Sync for RwLock<T> {}

impl<'a, T> !Send for RwLockReadGuard<'a, T> {}

impl<'a, T> !Send for RwLockWriteGuard<'a, T> {}

impl<T> RwLock<T> {
  /// Creates a new instance of an `RwLock<T>` which is unlocked.
  ///
  /// # Examples
  ///
  /// ```
  /// use drone::sync::RwLock;
  ///
  /// let lock = RwLock::new(5);
  /// ```
  pub const fn new(t: T) -> Self {
    Self {
      lock: AtomicUsize::new(NO_LOCK),
      data: UnsafeCell::new(t),
    }
  }

  /// Attempts to acquire this rwlock with shared read access.
  ///
  /// If the access could not be granted at this time, then `Err` is returned.
  /// Otherwise, an RAII guard is returned which will release the shared access
  /// when it is dropped.
  ///
  /// This function does not provide any guarantees with respect to the ordering
  /// of whether contentious readers or writers will acquire the lock first.
  ///
  /// # Examples
  ///
  /// ```
  /// use drone::sync::RwLock;
  ///
  /// let lock = RwLock::new(1);
  ///
  /// match lock.try_read() {
  ///   Some(n) => assert_eq!(*n, 1),
  ///   None => unreachable!(),
  /// };
  /// ```
  #[inline]
  pub fn try_read(&self) -> Option<RwLockReadGuard<T>> {
    loop {
      let current = self.lock.load(Relaxed);
      if current >= WRITE_LOCK - 1 {
        break None;
      }
      if self.lock.compare_and_swap(current, current + 1, Acquire) == current {
        break Some(RwLockReadGuard { lock: self });
      }
    }
  }

  /// Attempts to lock this rwlock with exclusive write access.
  ///
  /// If the lock could not be acquired at this time, then `Err` is returned.
  /// Otherwise, an RAII guard is returned which will release the lock when it
  /// is dropped.
  ///
  /// This function does not provide any guarantees with respect to the ordering
  /// of whether contentious readers or writers will acquire the lock first.
  ///
  /// # Examples
  ///
  /// ```
  /// use drone::sync::RwLock;
  ///
  /// let lock = RwLock::new(1);
  ///
  /// let n = lock.try_read().unwrap();
  /// assert_eq!(*n, 1);
  ///
  /// assert!(lock.try_write().is_none());
  /// ```
  #[inline]
  pub fn try_write(&self) -> Option<RwLockWriteGuard<T>> {
    if self.lock.compare_and_swap(NO_LOCK, WRITE_LOCK, Acquire) == NO_LOCK {
      Some(RwLockWriteGuard { lock: self })
    } else {
      None
    }
  }

  /// Consumes this `RwLock`, returning the underlying data.
  ///
  /// # Examples
  ///
  /// ```
  /// use drone::sync::RwLock;
  ///
  /// let lock = RwLock::new(String::new());
  /// {
  ///   let mut s = lock.try_write().unwrap();
  ///   *s = "modified".to_owned();
  /// }
  /// assert_eq!(lock.into_inner(), "modified");
  /// ```
  pub fn into_inner(self) -> T {
    let Self { data, .. } = self;
    unsafe { data.into_inner() }
  }

  /// Returns a mutable reference to the underlying data.
  ///
  /// Since this call borrows the `RwLock` mutably, no actual locking needs to
  /// take place --- the mutable borrow statically guarantees no locks exist.
  ///
  /// # Examples
  ///
  /// ```
  /// use drone::sync::RwLock;
  ///
  /// let mut lock = RwLock::new(0);
  /// *lock.get_mut() = 10;
  /// assert_eq!(*lock.try_read().unwrap(), 10);
  /// ```
  pub fn get_mut(&mut self) -> &mut T {
    unsafe { &mut *self.data.get() }
  }
}

impl<T: Default> Default for RwLock<T> {
  /// Creates a new `RwLock<T>`, with the `Default` value for T.
  #[inline]
  fn default() -> RwLock<T> {
    RwLock::new(Default::default())
  }
}

impl<'a, T> Deref for RwLockReadGuard<'a, T> {
  type Target = T;

  #[inline]
  fn deref(&self) -> &T {
    unsafe { &*self.lock.data.get() }
  }
}

impl<'a, T> Deref for RwLockWriteGuard<'a, T> {
  type Target = T;

  #[inline]
  fn deref(&self) -> &T {
    unsafe { &*self.lock.data.get() }
  }
}

impl<'a, T> DerefMut for RwLockWriteGuard<'a, T> {
  #[inline]
  fn deref_mut(&mut self) -> &mut T {
    unsafe { &mut *self.lock.data.get() }
  }
}

impl<'a, T> Drop for RwLockReadGuard<'a, T> {
  #[inline]
  fn drop(&mut self) {
    self.lock.lock.fetch_sub(1, Release);
  }
}

impl<'a, T> Drop for RwLockWriteGuard<'a, T> {
  #[inline]
  fn drop(&mut self) {
    self.lock.lock.store(NO_LOCK, Release);
  }
}
