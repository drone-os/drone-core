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

pub use self::{
  preempt::{current, with_preempted, PreemptedCell},
  tag::*,
  task::{current_task, init, TaskCell},
};

use crate::{
  fib::{Chain, FiberRoot},
  sv::SvOpt,
};

/// A thread interface.
pub trait Thread: Sized + Sync + 'static {
  /// Thread-local storage.
  type Local: ThreadLocal;

  /// Optional supervisor.
  type Sv: SvOpt;

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
  T: ThrTag,
{
  /// Thread.
  type Thr: Thread;

  /// Corresponding attachable thread token.
  type AThrToken: ThrToken<Att>;

  /// Corresponding triggerable thread token.
  type TThrToken: ThrToken<Ttt>;

  /// Corresponding controllable thread token.
  type CThrToken: ThrToken<Ctt>;

  /// Corresponding regular thread token.
  type RThrToken: ThrToken<Rtt>;

  /// A thread position within threads array.
  const THR_NUM: usize;

  /// Creates an instance of the thread token.
  ///
  /// # Safety
  ///
  /// Caller must take care for synchronizing instances.
  unsafe fn take() -> Self;

  /// Returns a reference to the thread.
  ///
  /// # Safety
  ///
  /// The method doesn't enforce privileges of the token.
  #[inline(always)]
  unsafe fn to_thr(self) -> &'static Self::Thr {
    get_thr::<Self, T>()
  }

  /// Converts to an attachable register token.
  #[inline(always)]
  fn to_attach(self) -> Self::AThrToken
  where
    T: ThrAttach,
  {
    unsafe { Self::AThrToken::take() }
  }

  /// Converts to a triggerable register token.
  #[inline(always)]
  fn to_trigger(self) -> Self::TThrToken
  where
    T: ThrTrigger,
  {
    unsafe { Self::TThrToken::take() }
  }

  /// Converts to a regular register token.
  #[inline(always)]
  fn to_regular(self) -> Self::RThrToken
  where
    T: ThrAttach + ThrTrigger,
  {
    unsafe { Self::RThrToken::take() }
  }

  /// Adds a new fiber to the thread.
  #[inline(always)]
  fn add_fib<F: FiberRoot>(self, fib: F)
  where
    T: ThrAttach,
  {
    unsafe { self.to_thr() }.fib_chain().add(fib);
  }

  /// Returns `true` if the fiber chain is empty.
  #[inline(always)]
  fn is_empty(self) -> bool {
    unsafe { self.to_thr() }.fib_chain().is_empty()
  }
}

/// A thread handler function.
///
/// # Safety
///
/// Must not be called concurrently.
pub unsafe fn thread_resume<T: ThrToken<U>, U: ThrTag>() {
  let thr = get_thr::<T, U>();
  with_preempted(thr.get_local().preempted(), T::THR_NUM, || {
    thr.fib_chain().drain();
  })
}

#[inline(always)]
unsafe fn get_thr<T: ThrToken<U>, U: ThrTag>() -> &'static T::Thr {
  &*T::Thr::first().add(T::THR_NUM)
}
