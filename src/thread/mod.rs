//! Interrupt-driven threads.
//!
//! # The threading model
//!
//! A Drone application consists of a static collection of threads. Each thread
//! consists of a dynamic chain of routines, which are executing sequentially
//! within a thread context.

mod chain;
mod routine_future;
mod stream_ring;
mod stream_unit;
mod task;
mod bindings;

pub use self::bindings::{ThreadBinding, ThreadBindings};
pub use self::chain::Chain;
pub use self::routine_future::RoutineFuture;
pub use self::task::{init, TaskCell};
pub use drone_core_macros::thread_local;

use self::stream_ring::{stream_ring, stream_ring_overwrite};
use self::stream_unit::stream_unit;
use sync::spsc::{ring, unit};

/// The index of the current thread.
static mut CURRENT_IDX: usize = 0;

/// Returns the index of the current thread.
#[inline(always)]
pub fn current_idx() -> usize {
  unsafe { CURRENT_IDX }
}

#[inline(always)]
unsafe fn set_current_idx(index: usize) {
  CURRENT_IDX = index;
}

/// Returns a static reference to the current thread.
#[inline(always)]
pub fn current<T: Thread>() -> &'static T {
  unsafe { (*T::array()).get_unchecked(current_idx()) }
}

/// A thread interface.
pub trait Thread: Sized + Sync + 'static {
  /// Returns a mutable pointer to the static array of threads.
  fn array() -> *mut [Self];

  /// Returns a reference to the routines chain.
  fn chain(&self) -> &Chain;

  /// Returns a mutable reference to the routines chain.
  fn chain_mut(&mut self) -> &mut Chain;

  /// Returns the cell for the task pointer.
  ///
  /// This method is safe because [`TaskCell`] doesn't have public API.
  ///
  /// [`TaskCell`]: struct.TaskCell.html
  fn task(&self) -> &TaskCell;

  /// Returns the index of the thread preempted by the current thread.
  fn preempted_idx(&self) -> usize;

  /// Sets the index of the thread preempted by the current thread.
  ///
  /// This method is safe because `&mut self` can't be obtained by public API.
  fn set_preempted_idx(&mut self, index: usize);

  /// Adds a new routine to the beginning of the chain. This method accepts a
  /// generator.
  #[inline]
  fn routine<G>(&self, g: G)
  where
    G: Generator<Yield = (), Return = ()>,
    G: Send + 'static,
  {
    self.chain().push(g);
  }

  /// Adds a new routine to the beginning of the chain. This method accepts a
  /// closure.
  #[inline]
  fn routine_fn<F>(&self, f: F)
  where
    F: FnOnce(),
    F: Send + 'static,
  {
    self.routine(|| {
      if false {
        yield;
      }
      f()
    });
  }

  /// Adds a new routine to the beginning of the chain. Returns a `Future` of
  /// the routine's return value. This method accepts a generator.
  #[inline]
  fn future<G, T, E>(&self, g: G) -> RoutineFuture<T, E>
  where
    G: Generator<Yield = (), Return = Result<T, E>>,
    G: Send + 'static,
    T: Send + 'static,
    E: Send + 'static,
  {
    RoutineFuture::new(self, g)
  }

  /// Adds a new routine to the beginning of the chain. Returns a `Future` of
  /// the routine's return value. This method accepts a closure.
  #[inline]
  fn future_fn<F, T, E>(&self, f: F) -> RoutineFuture<T, E>
  where
    F: FnOnce() -> Result<T, E>,
    F: Send + 'static,
    T: Send + 'static,
    E: Send + 'static,
  {
    self.future(|| {
      if false {
        yield;
      }
      f()
    })
  }

  /// Adds a new routine to the beginning of the chain. Returns a `Stream` of
  /// routine's yielded values. If `overflow` returns `Ok(())`, current value
  /// will be skipped. This method only accepts `()` as values.
  #[inline]
  fn stream<G, E, O>(&self, overflow: O, g: G) -> unit::Receiver<E>
  where
    G: Generator<Yield = Option<()>, Return = Result<Option<()>, E>>,
    O: Fn() -> Result<(), E>,
    G: Send + 'static,
    E: Send + 'static,
    O: Send + 'static,
  {
    stream_unit(self, g, overflow)
  }

  /// Adds a new routine to the beginning of the chain. Returns a `Stream` of
  /// routine's yielded values. Values will be skipped on overflow. This method
  /// only accepts `()` as values.
  #[inline]
  fn stream_skip<G, E>(&self, g: G) -> unit::Receiver<E>
  where
    G: Generator<Yield = Option<()>, Return = Result<Option<()>, E>>,
    G: Send + 'static,
    E: Send + 'static,
  {
    stream_unit(self, g, || Ok(()))
  }

  /// Adds a new routine to the beginning of the chain. Returns a `Stream` of
  /// routine's yielded values. If `overflow` returns `Ok(())`, currenct value
  /// will be skipped.
  #[inline]
  fn stream_ring<G, T, E, O>(
    &self,
    capacity: usize,
    overflow: O,
    g: G,
  ) -> ring::Receiver<T, E>
  where
    G: Generator<Yield = Option<T>, Return = Result<Option<T>, E>>,
    O: Fn(T) -> Result<(), E>,
    G: Send + 'static,
    T: Send + 'static,
    E: Send + 'static,
    O: Send + 'static,
  {
    stream_ring(self, capacity, g, overflow)
  }

  /// Adds a new routine to the beginning of the chain. Returns a `Stream` of
  /// routine's yielded values. New values will be skipped on overflow.
  #[inline]
  fn stream_ring_skip<G, T, E>(
    &self,
    capacity: usize,
    g: G,
  ) -> ring::Receiver<T, E>
  where
    G: Generator<Yield = Option<T>, Return = Result<Option<T>, E>>,
    G: Send + 'static,
    T: Send + 'static,
    E: Send + 'static,
  {
    stream_ring(self, capacity, g, |_| Ok(()))
  }

  /// Adds a new routine to the beginning of the chain. Returns a `Stream` of
  /// routine's yielded values. Old values will be overwritten on overflow.
  #[inline]
  fn stream_ring_overwrite<G, T, E>(
    &self,
    capacity: usize,
    g: G,
  ) -> ring::Receiver<T, E>
  where
    G: Generator<Yield = Option<T>, Return = Result<Option<T>, E>>,
    G: Send + 'static,
    T: Send + 'static,
    E: Send + 'static,
  {
    stream_ring_overwrite(self, capacity, g)
  }
}
