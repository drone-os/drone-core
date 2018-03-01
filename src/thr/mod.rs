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

use fib::Chain;

static mut CURRENT: usize = 0;

/// A thread interface.
pub trait Thread: Sized + Sync + 'static {
  /// Returns a mutable pointer to the static array of threads.
  fn all() -> *mut [Self];

  /// Returns a reference to the fibers stack.
  fn fib_chain(&self) -> &Chain;

  /// Returns a mutable reference to the fibers stack.
  fn fib_chain_mut(&mut self) -> &mut Chain;

  /// Returns the cell for the task pointer.
  fn task(&self) -> &TaskCell;

  /// Returns a mutable reference to the stored index of the preempted thread.
  fn preempted(&mut self) -> &mut usize;
}

/// Thread token.
pub trait ThrToken<T>
where
  Self: Sized + Clone + Copy,
  Self: Send + Sync + 'static,
  Self: AsRef<<Self as ThrToken<T>>::Thr>,
  T: ThrTag,
{
  /// Thread array.
  type Thr: Thread;

  /// A thread position within threads array.
  const THR_NUM: usize;

  /// A thread handler function, which should be passed to hardware.
  ///
  /// # Safety
  ///
  /// Must not be called concurrently.
  unsafe extern "C" fn handler() {
    let thr = (*Self::Thr::all()).get_unchecked_mut(Self::THR_NUM);
    *thr.preempted() = CURRENT;
    CURRENT = Self::THR_NUM;
    thr.fib_chain_mut().drain();
    CURRENT = *thr.preempted();
  }

  /// Returns a reference to the thread.
  #[inline(always)]
  fn as_thr(&self) -> &Self::Thr {
    unsafe { (*Self::Thr::all()).get_unchecked(Self::THR_NUM) }
  }
}

/// A set of thread tokens.
pub trait ThrTokens {
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
