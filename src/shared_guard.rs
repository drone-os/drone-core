//! A generic RAII guard with an ability to share between threads.

use core::{
  borrow::BorrowMut,
  marker::PhantomData,
  mem::forget,
  ops::{Deref, DerefMut},
  ptr,
};

/// A handler that is responsible to tearing down the guard.
pub trait GuardHandler<T> {
  /// Called on [`Guard`] drop.
  fn teardown(&mut self, data: &mut T);
}

/// An RAII scoped guard that runs [`GuardHandler::teardown`] on drop.
#[must_use]
pub struct Guard<'a, T, H: GuardHandler<T> + 'a> {
  data: &'a mut T,
  handler: H,
}

/// An RAII scoped guard that is produced by one of [`Guard`]'s `share` methods.
///
/// The instance should run the corresponding `merge` method eventually.
/// Otherwise it will panic on drop.
pub struct SharedGuard<'a, T, H: GuardHandler<T> + 'a, C> {
  data: &'a mut T,
  handler: H,
  _counter: PhantomData<C>,
}

/// A zero-sized token, instance of which guarantees that corresponding
/// [`GuardHandler::teardown`] method hasn't run.
pub struct GuardToken<H>(PhantomData<H>);

impl<'a, T, H: GuardHandler<T> + 'a> Guard<'a, T, H> {
  /// Creates a new [`Guard`].
  ///
  /// The caller should ensure that there is no other [`Guard`] exists or
  /// leaked.
  #[inline]
  pub fn new(data: &'a mut T, handler: H) -> Self {
    Self { data, handler }
  }
}

impl<H> GuardToken<H> {
  /// Creates a new [`GuardToken`].
  ///
  /// # Safety
  ///
  /// It can break guard guarantees.
  #[inline]
  pub unsafe fn new() -> Self {
    GuardToken(PhantomData)
  }

  /// Calls `f` with [`GuardToken`] reference.
  #[inline]
  pub fn with<'a, T, R>(
    _guard: impl BorrowMut<Guard<'a, T, H>>,
    f: impl FnOnce(&GuardToken<H>) -> R,
  ) -> R
  where
    T: 'a,
    H: GuardHandler<T> + 'a,
  {
    f(&GuardToken(PhantomData))
  }
}

impl<'a, T, H: GuardHandler<T> + 'a> Drop for Guard<'a, T, H> {
  #[inline]
  fn drop(&mut self) {
    self.handler.teardown(self.data);
  }
}

impl<'a, T, H: GuardHandler<T> + 'a, C> Drop for SharedGuard<'a, T, H, C> {
  #[inline]
  fn drop(&mut self) {
    panic!("Trying to drop SharedGuard which is in use");
  }
}

impl<'a, T, H: GuardHandler<T> + 'a> Deref for Guard<'a, T, H> {
  type Target = T;

  #[inline]
  fn deref(&self) -> &T {
    self.data
  }
}

impl<'a, T, H: GuardHandler<T> + 'a> DerefMut for Guard<'a, T, H> {
  #[inline]
  fn deref_mut(&mut self) -> &mut T {
    self.data
  }
}

impl<'a, T, H: GuardHandler<T> + 'a, C> Deref for SharedGuard<'a, T, H, C> {
  type Target = T;

  #[inline]
  fn deref(&self) -> &T {
    self.data
  }
}

impl<'a, T, H: GuardHandler<T> + 'a, C> DerefMut for SharedGuard<'a, T, H, C> {
  #[inline]
  fn deref_mut(&mut self) -> &mut T {
    self.data
  }
}

macro_rules! shared_counters {
  ($($name:ident,)*) => {
    $(
      /// A counter for [`SharedGuard`].
      pub struct $name;
    )*
  };
}

macro_rules! shared_guard {
  (
    $share:ident,
    $merge:ident,
    $counter:ident,
    $number:expr,
    $($data:expr),*
  ) => {
    impl<'a, T, H: GuardHandler<T> + 'a> Guard<'a, T, H> {
      /// Converts [`Guard`] into [`SharedGuard`] and an array of
      /// [`GuardToken`].
      #[inline]
      pub fn $share(
        self
      ) -> (SharedGuard<'a, T, H, $counter>, [GuardToken<H>; $number]) {
        let data = unsafe { ptr::read(&self.data) };
        let handler = unsafe { ptr::read(&self.handler) };
        forget(self);
        let guard = SharedGuard {
          data,
          handler,
          _counter: PhantomData,
        };
        (guard, [$(GuardToken($data)),*])
      }
    }

    impl<'a, T, H: GuardHandler<T> + 'a> SharedGuard<'a, T, H, $counter> {
      /// Converts [`SharedGuard`] and the array of [`GuardToken`] into
      /// [`Guard`].
      #[inline]
      pub fn $merge(self, tokens: [GuardToken<H>; $number]) -> Guard<'a, T, H> {
        let data = unsafe { ptr::read(&self.data) };
        let handler = unsafe { ptr::read(&self.handler) };
        forget(self);
        drop(tokens);
        Guard { data, handler }
      }
    }
  };
}

shared_counters! {
  Share1,
  Share2,
  Share3,
  Share4,
  Share5,
  Share6,
  Share7,
  Share8,
  Share9,
  Share10,
  Share11,
  Share12,
  Share13,
  Share14,
  Share15,
  Share16,
}

shared_guard!(share1, merge1, Share1, 1, PhantomData);
shared_guard!(share2, merge2, Share2, 2, PhantomData, PhantomData);
shared_guard!(
  share3,
  merge3,
  Share3,
  3,
  PhantomData,
  PhantomData,
  PhantomData
);
shared_guard!(
  share4,
  merge4,
  Share4,
  4,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData
);
shared_guard!(
  share5,
  merge5,
  Share5,
  5,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData
);
shared_guard!(
  share6,
  merge6,
  Share6,
  6,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData
);
shared_guard!(
  share7,
  merge7,
  Share7,
  7,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData
);
shared_guard!(
  share8,
  merge8,
  Share8,
  8,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData
);
shared_guard!(
  share9,
  merge9,
  Share9,
  9,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData
);
shared_guard!(
  share10,
  merge10,
  Share10,
  10,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData
);
shared_guard!(
  share11,
  merge11,
  Share11,
  11,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData
);
shared_guard!(
  share12,
  merge12,
  Share12,
  12,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData
);
shared_guard!(
  share13,
  merge13,
  Share13,
  13,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData
);
shared_guard!(
  share14,
  merge14,
  Share14,
  14,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData
);
shared_guard!(
  share15,
  merge15,
  Share15,
  15,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData
);
shared_guard!(
  share16,
  merge16,
  Share16,
  16,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData,
  PhantomData
);

#[cfg(test)]
mod tests {
  use super::*;
  use alloc::rc::Rc;
  use core::sync::atomic::{AtomicBool, Ordering::*};

  struct Data(Rc<AtomicBool>);

  struct Handler;

  impl Data {
    fn guard(&mut self, check: bool) -> Guard<'_, Self, Handler> {
      if check && self.0.load(Relaxed) {
        panic!("Another guard exists");
      }
      self.0.store(true, Relaxed);
      Guard::new(self, Handler)
    }
  }

  impl GuardHandler<Data> for Handler {
    fn teardown(&mut self, data: &mut Data) {
      data.0.store(false, Relaxed);
    }
  }

  #[test]
  fn in_place() {
    let counter = Rc::new(AtomicBool::new(false));
    let mut data = Data(Rc::clone(&counter));
    let guard = data.guard(true);
    assert!(counter.load(Relaxed));
    drop(guard);
    assert!(!counter.load(Relaxed));
  }

  #[test]
  fn shared() {
    let counter = Rc::new(AtomicBool::new(false));
    let mut data = Data(Rc::clone(&counter));
    let guard = data.guard(true);
    assert!(counter.load(Relaxed));
    let (guard, [token]) = guard.share1();
    assert!(counter.load(Relaxed));
    let guard = guard.merge1([token]);
    assert!(counter.load(Relaxed));
    drop(guard);
    assert!(!counter.load(Relaxed));
  }

  #[test]
  #[should_panic]
  fn shared_drop() {
    let mut data = Data(Rc::new(AtomicBool::new(false)));
    let guard = data.guard(true);
    let (_guard, [_token]) = guard.share1();
  }

  #[test]
  fn improper_impl() {
    let counter = Rc::new(AtomicBool::new(false));
    let mut data = Data(Rc::clone(&counter));
    let guard = data.guard(true);
    let (guard, [token1]) = guard.share1();
    forget(guard);
    assert!(counter.load(Relaxed));
    // In a proper implementation this should fail.
    let guard = data.guard(false);
    let (guard, [token2]) = guard.share1();
    let guard = guard.merge1([token1]);
    drop(guard);
    assert!(!counter.load(Relaxed));
    // Guard is dropped but something can still rely on the existance of token2.
    drop(token2);
  }

  #[test]
  #[should_panic]
  fn proper_impl() {
    let counter = Rc::new(AtomicBool::new(false));
    let mut data = Data(Rc::clone(&counter));
    let guard = data.guard(true);
    let (guard, [_token]) = guard.share1();
    forget(guard);
    assert!(counter.load(Relaxed));
    let _ = data.guard(true);
  }
}
