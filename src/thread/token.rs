use super::CURRENT;
use core::ops::Deref;
use thread::prelude::*;

/// Thread token.
pub trait ThreadToken<T>
where
  Self: Sized + Clone + Copy,
  Self: Send + Sync + 'static,
  Self: Deref<Target = <Self as ThreadToken<T>>::Thread>,
  T: ThreadTag,
{
  /// Thread array.
  type Thread: Thread;

  /// A thread position within threads array.
  const THREAD_NUMBER: usize;

  /// A thread handler function, which should be passed to hardware.
  ///
  /// # Safety
  ///
  /// Must not be called concurrently.
  unsafe extern "C" fn handler() {
    let thread = (*Self::Thread::all()).get_unchecked_mut(Self::THREAD_NUMBER);
    *thread.preempted() = CURRENT;
    CURRENT = Self::THREAD_NUMBER;
    thread.fibers_mut().drain();
    CURRENT = *thread.preempted();
  }

  /// Returns a reference to the thread.
  #[inline(always)]
  fn as_thread(&self) -> &Self::Thread {
    unsafe { (*Self::Thread::all()).get_unchecked(Self::THREAD_NUMBER) }
  }
}

/// A set of thread tokens.
pub trait ThreadTokens {
  /// Creates a new set of thread tokens.
  ///
  /// # Safety
  ///
  /// Must be called no more than once.
  unsafe fn new() -> Self;
}
