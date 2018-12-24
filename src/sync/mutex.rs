use core::{
  cell::UnsafeCell,
  ops::{Deref, DerefMut},
  sync::atomic::{AtomicBool, Ordering::*},
};

/// A mutual exclusion primitive useful for protecting shared data.
///
/// This mutex supports only [`try_lock`](Mutex::try_lock) method, and hence
/// never blocks.
pub struct Mutex<T> {
  lock: AtomicBool,
  data: UnsafeCell<T>,
}

/// An RAII implementation of a "scoped lock" of a mutex. When this structure is
/// dropped (falls out of scope), the lock will be unlocked.
///
/// The data protected by the mutex can be accessed through this guard via its
/// `Deref` and `DerefMut` implementations.
///
/// This structure is created by the [`try_lock`](Mutex::try_lock) method on
/// [`Mutex`](Mutex).
#[must_use]
pub struct MutexGuard<'a, T: 'a> {
  lock: &'a Mutex<T>,
}

unsafe impl<T: Send> Send for Mutex<T> {}
unsafe impl<T: Send> Sync for Mutex<T> {}

impl<'a, T> !Send for MutexGuard<'a, T> {}
unsafe impl<'a, T: Sync> Sync for MutexGuard<'a, T> {}

impl<T> Mutex<T> {
  /// Creates a new mutex in an unlocked state ready for use.
  ///
  /// # Examples
  ///
  /// ```
  /// use drone_core::sync::Mutex;
  ///
  /// let mutex = Mutex::new(0);
  /// ```
  #[inline]
  pub const fn new(t: T) -> Self {
    Self {
      lock: AtomicBool::new(false),
      data: UnsafeCell::new(t),
    }
  }

  /// Attempts to acquire this lock.
  ///
  /// If the lock could not be acquired at this time, then `None` is returned.
  /// Otherwise, a RAII guard is returned. The lock will be unlocked when the
  /// guard is dropped.
  ///
  /// # Examples
  ///
  /// ```
  /// use drone_core::sync::Mutex;
  ///
  /// let mutex = Mutex::new(1);
  ///
  /// match mutex.try_lock() {
  ///   Some(n) => assert_eq!(*n, 1),
  ///   None => unreachable!(),
  /// };
  /// ```
  #[inline]
  pub fn try_lock(&self) -> Option<MutexGuard<T>> {
    if self.lock.swap(true, Acquire) {
      None
    } else {
      Some(MutexGuard { lock: self })
    }
  }

  /// Consumes this mutex, returning the underlying data.
  ///
  /// # Examples
  ///
  /// ```
  /// use drone_core::sync::Mutex;
  ///
  /// let mutex = Mutex::new(0);
  /// assert_eq!(mutex.into_inner(), 0);
  /// ```
  #[inline]
  pub fn into_inner(self) -> T {
    self.data.into_inner()
  }

  /// Returns a mutable reference to the underlying data.
  ///
  /// Since this call borrows the `Mutex` mutably, no actual locking needs to
  /// take place --- the mutable borrow statically guarantees no locks exist.
  ///
  /// # Examples
  ///
  /// ```
  /// use drone_core::sync::Mutex;
  ///
  /// let mut mutex = Mutex::new(0);
  /// *mutex.get_mut() = 10;
  /// assert_eq!(*mutex.try_lock().unwrap(), 10);
  /// ```
  #[inline]
  pub fn get_mut(&mut self) -> &mut T {
    unsafe { &mut *self.data.get() }
  }
}

impl<T: Default> Default for Mutex<T> {
  /// Creates a `Mutex<T>`, with the `Default` value for T.
  #[inline]
  fn default() -> Self {
    Self::new(Default::default())
  }
}

impl<'a, T> Deref for MutexGuard<'a, T> {
  type Target = T;

  #[inline]
  fn deref(&self) -> &T {
    unsafe { &*self.lock.data.get() }
  }
}

impl<'a, T> DerefMut for MutexGuard<'a, T> {
  #[inline]
  fn deref_mut(&mut self) -> &mut T {
    unsafe { &mut *self.lock.data.get() }
  }
}

impl<'a, T> Drop for MutexGuard<'a, T> {
  #[inline]
  fn drop(&mut self) {
    self.lock.lock.store(false, Release);
  }
}
