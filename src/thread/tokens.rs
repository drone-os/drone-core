use super::{current_idx, set_current_idx};
use core::marker::PhantomData;
use core::ops::Deref;
use thread::prelude::*;

/// Thread token.
pub struct ThreadToken<T: Thread, U: ThreadNumber> {
  _thread: PhantomData<&'static T>,
  _token: PhantomData<&'static U>,
}

/// Thread token.
pub trait ThreadNumber: Sync + 'static {
  /// A thread position within threads array.
  const THREAD_NUMBER: usize;
}

/// A set of thread tokens.
pub trait ThreadTokens<T: Thread> {
  /// Creates a new set of thread tokens.
  ///
  /// # Safety
  ///
  /// * Must be called no more than once.
  /// * Must be called at the very beginning of the program flow.
  unsafe fn new() -> Self;
}

#[cfg_attr(feature = "clippy", allow(new_without_default_derive))]
impl<T: Thread, U: ThreadNumber> ThreadToken<T, U> {
  /// Creates a new `ThreadToken`.
  ///
  /// # Safety
  ///
  /// * Must be called no more than once.
  /// * Must be called at the very beginning of the program flow.
  #[inline(always)]
  pub unsafe fn new() -> Self {
    Self {
      _thread: PhantomData,
      _token: PhantomData,
    }
  }

  /// A thread handler function, which should be passed to hardware.
  ///
  /// # Safety
  ///
  /// Must not be called concurrently.
  pub unsafe extern "C" fn handler() {
    let thread = Self::get_mut();
    thread.set_preempted_idx(current_idx());
    set_current_idx(U::THREAD_NUMBER);
    thread.chain_mut().drain();
    set_current_idx(thread.preempted_idx());
  }

  /// Returns a mutable reference to the thread.
  ///
  /// # Safety
  ///
  /// Caller must ensure that the access is unique.
  #[inline(always)]
  pub unsafe fn get_mut() -> &'static mut T {
    (*T::array()).get_unchecked_mut(U::THREAD_NUMBER)
  }

  /// Returns a static reference to the thread.
  #[inline(always)]
  pub fn as_thread(&self) -> &'static T {
    unsafe { (*T::array()).get_unchecked(U::THREAD_NUMBER) }
  }
}

#[cfg_attr(feature = "clippy", allow(expl_impl_clone_on_copy))]
impl<T: Thread, U: ThreadNumber> Clone for ThreadToken<T, U> {
  fn clone(&self) -> Self {
    unsafe { Self::new() }
  }
}

impl<T: Thread, U: ThreadNumber> Copy for ThreadToken<T, U> {}

impl<T: Thread, U: ThreadNumber> Deref for ThreadToken<T, U> {
  type Target = T;

  #[inline(always)]
  fn deref(&self) -> &T {
    self.as_thread()
  }
}
