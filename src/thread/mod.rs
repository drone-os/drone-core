//! Interrupt-driven threads.
//!
//! # The threading model
//!
//! A Drone application consists of a static collection of threads. Each thread
//! consists of a dynamic stack of routines, which are executing sequentially
//! within a thread context.

pub mod prelude;

mod routine;
mod tag;
mod task;
mod token;
mod tokens;

pub use self::routine::{RoutineFuture, RoutineStack, RoutineStreamRing,
                        RoutineStreamUnit};
pub use self::tag::*;
pub use self::task::{init, TaskCell};
pub use self::token::ThreadToken;
pub use self::tokens::ThreadTokens;
pub use drone_core_macros::thread_local;

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

  /// Returns a reference to the routines stack.
  fn routines(&self) -> &RoutineStack;

  /// Returns a mutable reference to the routines stack.
  fn routines_mut(&mut self) -> &mut RoutineStack;

  /// Returns the cell for the task pointer.
  fn task(&self) -> &TaskCell;

  /// Returns a mutable reference to the stored index of the preempted thread.
  fn preempted(&mut self) -> &mut usize;

  /// Adds a new routine to the stack. This method accepts a generator.
  #[inline(always)]
  fn routine<G>(&self, g: G)
  where
    G: Generator<Yield = (), Return = ()>,
    G: Send + 'static,
  {
    self.routines().push(g);
  }

  /// Adds a new routine to the stack. This method accepts a closure.
  #[inline(always)]
  fn routine_fn<F>(&self, f: F)
  where
    F: FnOnce(),
    F: Send + 'static,
  {
    self.routines().push(|| {
      if false {
        yield;
      }
      f()
    });
  }

  /// Adds a new routine to the stack. Returns a `Future` of the routine's
  /// return value. This method accepts a generator.
  #[inline(always)]
  fn future<G, T, E>(&self, g: G) -> RoutineFuture<T, E>
  where
    G: Generator<Yield = (), Return = Result<T, E>>,
    G: Send + 'static,
    T: Send + 'static,
    E: Send + 'static,
  {
    RoutineFuture::new(self, g)
  }

  /// Adds a new routine to the stack. Returns a `Future` of the routine's
  /// return value. This method accepts a closure.
  #[inline(always)]
  fn future_fn<F, T, E>(&self, f: F) -> RoutineFuture<T, E>
  where
    F: FnOnce() -> Result<T, E>,
    F: Send + 'static,
    T: Send + 'static,
    E: Send + 'static,
  {
    RoutineFuture::new(self, || {
      if false {
        yield;
      }
      f()
    })
  }

  /// Adds a new routine to the stack. Returns a `Stream` of routine's yielded
  /// values. If `overflow` returns `Ok(())`, current value will be skipped.
  /// This method only accepts `()` as values.
  #[inline(always)]
  fn stream<G, E, O>(&self, overflow: O, g: G) -> RoutineStreamUnit<E>
  where
    G: Generator<Yield = Option<()>, Return = Result<Option<()>, E>>,
    O: Fn() -> Result<(), E>,
    G: Send + 'static,
    E: Send + 'static,
    O: Send + 'static,
  {
    RoutineStreamUnit::new(self, g, overflow)
  }

  /// Adds a new routine to the stack. Returns a `Stream` of routine's yielded
  /// values. Values will be skipped on overflow. This method only accepts `()`
  /// as values.
  #[inline(always)]
  fn stream_skip<G, E>(&self, g: G) -> RoutineStreamUnit<E>
  where
    G: Generator<Yield = Option<()>, Return = Result<Option<()>, E>>,
    G: Send + 'static,
    E: Send + 'static,
  {
    RoutineStreamUnit::new(self, g, || Ok(()))
  }

  /// Adds a new routine to the stack. Returns a `Stream` of routine's yielded
  /// values. If `overflow` returns `Ok(())`, currenct value will be skipped.
  #[inline(always)]
  fn stream_ring<G, T, E, O>(
    &self,
    capacity: usize,
    overflow: O,
    g: G,
  ) -> RoutineStreamRing<T, E>
  where
    G: Generator<Yield = Option<T>, Return = Result<Option<T>, E>>,
    O: Fn(T) -> Result<(), E>,
    G: Send + 'static,
    T: Send + 'static,
    E: Send + 'static,
    O: Send + 'static,
  {
    RoutineStreamRing::new(self, capacity, g, overflow)
  }

  /// Adds a new routine to the stack. Returns a `Stream` of routine's yielded
  /// values. New values will be skipped on overflow.
  #[inline(always)]
  fn stream_ring_skip<G, T, E>(
    &self,
    capacity: usize,
    g: G,
  ) -> RoutineStreamRing<T, E>
  where
    G: Generator<Yield = Option<T>, Return = Result<Option<T>, E>>,
    G: Send + 'static,
    T: Send + 'static,
    E: Send + 'static,
  {
    RoutineStreamRing::new(self, capacity, g, |_| Ok(()))
  }

  /// Adds a new routine to the stack. Returns a `Stream` of routine's yielded
  /// values. Old values will be overwritten on overflow.
  #[inline(always)]
  fn stream_ring_overwrite<G, T, E>(
    &self,
    capacity: usize,
    g: G,
  ) -> RoutineStreamRing<T, E>
  where
    G: Generator<Yield = Option<T>, Return = Result<Option<T>, E>>,
    G: Send + 'static,
    T: Send + 'static,
    E: Send + 'static,
  {
    RoutineStreamRing::new_overwrite(self, capacity, g)
  }
}
