//! Interrupt-driven threads.
//!
//! # The threading model
//!
//! A Drone application consists of a static collection of threads. Each thread
//! consists of a dynamic stack of fibers, which are executing sequentially
//! within a thread context.

pub mod prelude;

mod tag;
mod task;

pub use self::tag::*;
pub use self::task::{init, TaskCell};

use fiber::Chain;

static mut CURRENT: usize = 0;

/// A thread interface.
pub trait Thread: Sized + Sync + 'static {
  /// Returns a mutable pointer to the static array of threads.
  fn all() -> *mut [Self];

  /// Returns a reference to the fibers stack.
  fn fibers(&self) -> &Chain;

  /// Returns a mutable reference to the fibers stack.
  fn fibers_mut(&mut self) -> &mut Chain;

  /// Returns the cell for the task pointer.
  fn task(&self) -> &TaskCell;

  /// Returns a mutable reference to the stored index of the preempted thread.
  fn preempted(&mut self) -> &mut usize;
}

/// Thread token.
pub trait ThdToken<T>
where
  Self: Sized + Clone + Copy,
  Self: Send + Sync + 'static,
  Self: AsRef<<Self as ThdToken<T>>::Thd>,
  T: ThdTag,
{
  /// Thread array.
  type Thd: Thread;

  /// A thread position within threads array.
  const THD_NUM: usize;

  /// A thread handler function, which should be passed to hardware.
  ///
  /// # Safety
  ///
  /// Must not be called concurrently.
  unsafe extern "C" fn handler() {
    let thread = (*Self::Thd::all()).get_unchecked_mut(Self::THD_NUM);
    *thread.preempted() = CURRENT;
    CURRENT = Self::THD_NUM;
    thread.fibers_mut().drain();
    CURRENT = *thread.preempted();
  }

  /// Returns a reference to the thread.
  #[inline(always)]
  fn as_thd(&self) -> &Self::Thd {
    unsafe { (*Self::Thd::all()).get_unchecked(Self::THD_NUM) }
  }
}

/// A set of thread tokens.
pub trait ThdTokens {
  /// Creates a new set of thread tokens.
  ///
  /// # Safety
  ///
  /// Must be called no more than once.
  unsafe fn new() -> Self;
}

/// Returns a static reference to the current thread.
#[inline(always)]
pub fn current<T: Thread>() -> &'static T {
  unsafe { (*T::all()).get_unchecked(CURRENT) }
}
