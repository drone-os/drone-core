use super::{current_idx, set_current_idx};
use core::ops::Deref;

/// Thread binding.
pub trait ThreadBinding<T>
where
  Self: Copy + Send + Sync + 'static,
  Self: Deref<Target = T>,
  T: Thread,
{
  /// A thread position within threads array.
  const INDEX: usize;

  /// A thread handler function, which should be passed to hardware.
  ///
  /// # Safety
  ///
  /// Must not be called concurrently.
  unsafe extern "C" fn handler() {
    let thread = Self::get_mut();
    thread.set_preempted_idx(current_idx());
    set_current_idx(Self::INDEX);
    thread.chain_mut().drain();
    set_current_idx(thread.preempted_idx());
  }

  /// Creates a new thread binding.
  ///
  /// # Safety
  ///
  /// * Must be called no more than once.
  /// * Must be called at the very beginning of the program flow.
  unsafe fn bind() -> Self;

  /// Returns a mutable reference to the thread.
  ///
  /// # Safety
  ///
  /// Caller must ensure that the access is unique.
  #[inline(always)]
  unsafe fn get_mut() -> &'static mut T {
    (*T::array()).get_unchecked_mut(Self::INDEX)
  }

  /// Returns a static reference to the thread.
  #[inline(always)]
  fn as_thread(&self) -> &'static T {
    unsafe { (*T::array()).get_unchecked(Self::INDEX) }
  }
}

/// A set of thread bindings.
pub trait ThreadBindings {
  /// Creates a new set of thread bindings.
  ///
  /// # Safety
  ///
  /// * Must be called no more than once.
  /// * Must be called at the very beginning of the program flow.
  unsafe fn new() -> Self;
}
