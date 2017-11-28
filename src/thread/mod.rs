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

pub use self::chain::Chain;
pub use self::routine_future::RoutineFuture;
pub use drone_macros::thread_local;

use self::stream_ring::{stream_ring, stream_ring_overwrite};
use self::stream_unit::stream_unit;
use core::cell::UnsafeCell;
use core::ptr;
use futures::task;
use sync::spsc::{ring, unit};

/// The pointer to the current running thread.
static mut CURRENT_ID: usize = 0;

/// A thread-local storage of the task pointer.
pub struct TaskCell(UnsafeCell<*mut u8>);

/// Returns the id of the thread that invokes it.
#[inline(always)]
pub fn current_id() -> usize {
  unsafe { CURRENT_ID }
}

#[inline(always)]
unsafe fn set_current_id(id: usize) {
  CURRENT_ID = id;
}

/// Returns a reference to the thread that invokes it.
#[inline(always)]
pub fn current<T>() -> &'static T
where
  T: Thread,
{
  unsafe { T::get_unchecked(current_id()) }
}

/// Initialize the `futures` task system.
///
/// # Safety
///
/// Must be called before using `futures`.
#[inline(always)]
pub unsafe fn init<T>() -> bool
where
  T: Thread + 'static,
{
  task::init(get_task::<T>, set_task::<T>)
}

fn get_task<T>() -> *mut u8
where
  T: Thread + 'static,
{
  unsafe { current::<T>().task().get() }
}

fn set_task<T>(task: *mut u8)
where
  T: Thread + 'static,
{
  unsafe { current::<T>().task().set(task) };
}

/// A thread interface.
pub trait Thread: Sized + Sync {
  /// Returns a reference to a thread by its `id`.
  ///
  /// # Safety
  ///
  /// `id` must be a valid index.
  unsafe fn get_unchecked(id: usize) -> &'static Self;

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

  /// Returns the id of the thread preempted by the current thread.
  fn preempted_id(&self) -> usize;

  /// Sets the id of the thread preempted by the current thread.
  ///
  /// This method is safe because `&mut self` can't be obtained by public API.
  ///
  /// [`resume`]: #method.resume
  fn set_preempted_id(&mut self, id: usize);

  /// Resumes associated routines sequentially.
  ///
  /// Completed routines will be dropped.
  ///
  /// # Safety
  ///
  /// `id` must be the index of the thread.
  #[inline(always)]
  unsafe fn resume(&mut self, id: usize) {
    self.set_preempted_id(current_id());
    set_current_id(id);
    self.chain_mut().drain();
    set_current_id(self.preempted_id());
  }

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

impl TaskCell {
  /// Creates a new `TaskCell`.
  #[inline(always)]
  pub const fn new() -> Self {
    TaskCell(UnsafeCell::new(ptr::null_mut()))
  }

  #[inline(always)]
  unsafe fn get(&self) -> *mut u8 {
    *self.0.get()
  }

  #[inline(always)]
  unsafe fn set(&self, task: *mut u8) {
    *self.0.get() = task;
  }
}

unsafe impl Sync for TaskCell {}
