use super::CURRENT;
use core::marker::PhantomData;
use core::ops::Deref;
use thread::prelude::*;

/// Thread token.
pub struct ThreadToken<T: Thread, U: ThreadNumber> {
  _thread: PhantomData<&'static T>,
  _token: PhantomData<&'static U>,
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
    let thread = (*T::all()).get_unchecked_mut(U::THREAD_NUMBER);
    *thread.preempted() = CURRENT;
    CURRENT = U::THREAD_NUMBER;
    thread.routines_mut().drain();
    CURRENT = *thread.preempted();
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
    unsafe { (*T::all()).get_unchecked(U::THREAD_NUMBER) }
  }
}
