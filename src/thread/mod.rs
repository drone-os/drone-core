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
mod token;

pub use self::tag::*;
pub use self::task::{init, TaskCell};
pub use self::token::{ThreadToken, ThreadTokens};
pub use drone_core_macros::thread_local;

use fiber::{FiberFuture, FiberStreamRing, FiberStreamUnit, Fibers};

/// An index of the current thread.
static mut CURRENT: usize = 0;

/// Returns a static reference to the current thread.
#[inline(always)]
pub fn current<T: Thread>() -> &'static T {
  unsafe { (*T::all()).get_unchecked(CURRENT) }
}

/// A thread interface.
pub trait Thread: Sized + Sync + 'static {
  /// Returns a mutable pointer to the static array of threads.
  fn all() -> *mut [Self];

  /// Returns a reference to the fibers stack.
  fn fibers(&self) -> &Fibers;

  /// Returns a mutable reference to the fibers stack.
  fn fibers_mut(&mut self) -> &mut Fibers;

  /// Returns the cell for the task pointer.
  fn task(&self) -> &TaskCell;

  /// Returns a mutable reference to the stored index of the preempted thread.
  fn preempted(&mut self) -> &mut usize;

  /// Adds a new fiber to the stack. This method accepts a generator.
  #[inline(always)]
  fn fiber<G>(&self, gen: G)
  where
    G: Generator<Yield = (), Return = ()>,
    G: Send + 'static,
  {
    self.fibers().add(gen);
  }

  /// Adds a new fiber to the stack. This method accepts a closure.
  #[inline(always)]
  fn fiber_fn<F>(&self, f: F)
  where
    F: FnOnce(),
    F: Send + 'static,
  {
    self.fibers().add(|| {
      if false {
        yield;
      }
      f()
    });
  }

  /// Adds a new fiber to the stack. Returns a `Future` of the fiber's return
  /// value. This method accepts a generator.
  #[inline(always)]
  fn future<G, R, E>(&self, gen: G) -> FiberFuture<R, E>
  where
    G: Generator<Yield = (), Return = Result<R, E>>,
    G: Send + 'static,
    R: Send + 'static,
    E: Send + 'static,
  {
    FiberFuture::new(self, gen)
  }

  /// Adds a new fiber to the stack. Returns a `Future` of the fiber's return
  /// value. This method accepts a closure.
  #[inline(always)]
  fn future_fn<F, R, E>(&self, f: F) -> FiberFuture<R, E>
  where
    F: FnOnce() -> Result<R, E>,
    F: Send + 'static,
    R: Send + 'static,
    E: Send + 'static,
  {
    FiberFuture::new(self, || {
      if false {
        yield;
      }
      f()
    })
  }

  /// Adds a new fiber to the stack. Returns a `Stream` of fiber's yielded
  /// values. If `overflow` returns `Ok(())`, current value will be skipped.
  /// This method only accepts `()` as values.
  #[inline(always)]
  fn stream<G, E, O>(&self, overflow: O, gen: G) -> FiberStreamUnit<E>
  where
    G: Generator<Yield = Option<()>, Return = Result<Option<()>, E>>,
    O: Fn() -> Result<(), E>,
    G: Send + 'static,
    E: Send + 'static,
    O: Send + 'static,
  {
    FiberStreamUnit::new(self, gen, overflow)
  }

  /// Adds a new fiber to the stack. Returns a `Stream` of fiber's yielded
  /// values. Values will be skipped on overflow. This method only accepts `()`
  /// as values.
  #[inline(always)]
  fn stream_skip<G, E>(&self, gen: G) -> FiberStreamUnit<E>
  where
    G: Generator<Yield = Option<()>, Return = Result<Option<()>, E>>,
    G: Send + 'static,
    E: Send + 'static,
  {
    FiberStreamUnit::new(self, gen, || Ok(()))
  }

  /// Adds a new fiber to the stack. Returns a `Stream` of fiber's yielded
  /// values. If `overflow` returns `Ok(())`, currenct value will be skipped.
  #[inline(always)]
  fn stream_ring<G, R, E, O>(
    &self,
    capacity: usize,
    overflow: O,
    gen: G,
  ) -> FiberStreamRing<R, E>
  where
    G: Generator<Yield = Option<R>, Return = Result<Option<R>, E>>,
    O: Fn(R) -> Result<(), E>,
    G: Send + 'static,
    R: Send + 'static,
    E: Send + 'static,
    O: Send + 'static,
  {
    FiberStreamRing::new(self, capacity, gen, overflow)
  }

  /// Adds a new fiber to the stack. Returns a `Stream` of fiber's yielded
  /// values. New values will be skipped on overflow.
  #[inline(always)]
  fn stream_ring_skip<G, R, E>(
    &self,
    capacity: usize,
    gen: G,
  ) -> FiberStreamRing<R, E>
  where
    G: Generator<Yield = Option<R>, Return = Result<Option<R>, E>>,
    G: Send + 'static,
    R: Send + 'static,
    E: Send + 'static,
  {
    FiberStreamRing::new(self, capacity, gen, |_| Ok(()))
  }

  /// Adds a new fiber to the stack. Returns a `Stream` of fiber's yielded
  /// values. Old values will be overwritten on overflow.
  #[inline(always)]
  fn stream_ring_overwrite<G, R, E>(
    &self,
    capacity: usize,
    gen: G,
  ) -> FiberStreamRing<R, E>
  where
    G: Generator<Yield = Option<R>, Return = Result<Option<R>, E>>,
    G: Send + 'static,
    R: Send + 'static,
    E: Send + 'static,
  {
    FiberStreamRing::new_overwrite(self, capacity, gen)
  }
}
