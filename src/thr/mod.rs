//! Interrupt-driven threads.
//!
//! # The threading model
//!
//! A Drone application consists of a static collection of threads. Each thread
//! consists of a dynamic stack of fibers, which are executing sequentially
//! within a thread context.

pub mod prelude;

mod preempt;
mod tag;
mod task;

pub use self::preempt::{current, with_preempted, PreemptedCell};
pub use self::tag::*;
pub use self::task::{__current_task, init, TaskCell};

use fib::Chain;
use sv::Supervisor;

/// A thread interface.
pub trait Thread: Sized + Sync + 'static {
  /// Thread-local storage.
  type Local: ThreadLocal;

  /// Supervisor.
  type Sv: Supervisor;

  /// Returns a pointer to the first thread.
  fn first() -> *const Self;

  /// Returns a reference to the fibers stack.
  fn fib_chain(&self) -> &Chain;

  /// Returns a thread-local storage. A safe way to get it is via
  /// [`current`](current).
  ///
  /// # Safety
  ///
  /// Must be called only if the current thread is active.
  unsafe fn get_local(&self) -> &Self::Local;
}

/// A thread-local storage.
pub trait ThreadLocal: Sized + 'static {
  /// Returns the cell for the current task context.
  fn task(&self) -> &TaskCell;

  /// Returns a mutable reference to the stored index of the preempted thread.
  fn preempted(&self) -> &PreemptedCell;
}

/// Thread token.
pub trait ThrToken<T>
where
  Self: Sized + Clone + Copy,
  Self: Send + Sync + 'static,
  Self: AsRef<<Self as ThrToken<T>>::Thr>,
  T: ThrTag,
{
  /// Thread.
  type Thr: Thread;

  /// A thread position within threads array.
  const THR_NUM: usize;

  /// Returns a reference to the thread.
  #[inline(always)]
  fn get_thr() -> &'static Self::Thr {
    unsafe { &*Self::Thr::first().add(Self::THR_NUM) }
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

/// A thread handler function.
///
/// # Safety
///
/// Must not be called concurrently.
pub unsafe fn thread_resume<T: ThrToken<U>, U: ThrTag>() {
  let thr = T::get_thr();
  with_preempted(thr.get_local().preempted(), T::THR_NUM, || {
    thr.fib_chain().drain();
  })
}
