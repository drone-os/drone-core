//! Interrupt-driven threads.
//!
//! # The threading model
//!
//! A Drone application consists of a static collection of threads. Each thread
//! consists of a dynamic list of routines, which are executing sequentially
//! within a thread context.

pub mod routine;

pub use self::routine::Routine;
pub use drone_macros::thread_local_imp;

use collections::LinkedList;
use core::ops::Generator;
use task::ThreadFuture;

/// A pointer to the current running thread.
static mut CURRENT_ID: usize = 0;

/// Returns the id of the thread that invokes it.
pub fn current_id() -> usize {
  unsafe { CURRENT_ID }
}

/// Sets the id of the current thread.
///
/// # Safety
///
/// Calling this outside Drone internals is unpredictable.
unsafe fn set_current_id(id: usize) {
  CURRENT_ID = id;
}

/// Returns a reference to the thread that invokes it.
pub fn current<T>() -> &'static T
where
  T: Thread,
{
  unsafe { T::get_unchecked(current_id()) }
}

/// A thread interface.
pub trait Thread: Sized {
  /// Returns a reference to a thread by `id`.
  unsafe fn get_unchecked(id: usize) -> &'static Self;

  /// Provides a reference to the list of routines.
  fn list(&self) -> &LinkedList<Routine>;

  /// Provides a mutable reference to the list of routines.
  fn list_mut(&mut self) -> &mut LinkedList<Routine>;

  /// Returns the id of the thread preempted by the current one.
  fn preempted_id(&self) -> usize;

  /// Saves the id of the thread preempted by the current one.
  ///
  /// # Safety
  ///
  /// Calling this outside Drone internals is unpredictable.
  unsafe fn set_preempted_id(&mut self, id: usize);

  /// Returns the current thread-local value of the task system's pointer.
  fn task(&self) -> *mut u8;

  /// Sets the current thread-local value of the task system's pointer.
  ///
  /// # Safety
  ///
  /// Calling this outside Drone internals is unpredictable.
  unsafe fn set_task(&self, task: *mut u8);

  /// Runs associated routines sequentially.
  ///
  /// Completed routines are dropped.
  ///
  /// # Safety
  ///
  /// Must not be called concurrently.
  unsafe fn run(&mut self, id: usize) {
    self.set_preempted_id(current_id());
    set_current_id(id);
    self
      .list_mut()
      .drain_filter(Routine::resume)
      .for_each(|_| {});
    set_current_id(self.preempted_id());
  }

  /// Spawns a new generator within the thread.
  fn spawn<G>(&self, g: G)
  where
    G: Generator<Yield = (), Return = ()>,
    G: Send + 'static,
  {
    self.list().push(g.into());
  }

  /// Spawns a new closure within the thread.
  fn spawn_fn<F>(&self, f: F)
  where
    F: FnOnce(),
    F: Send + 'static,
  {
    self.spawn(|| {
      if false {
        yield;
      }
      f()
    });
  }

  /// Spawns a new future within the thread.
  fn future<G, R, E>(&self, g: G) -> ThreadFuture<R, E>
  where
    G: Generator<Yield = (), Return = Result<R, E>>,
    G: Send + 'static,
    R: Send + 'static,
    E: Send + 'static,
  {
    ThreadFuture::new(self, g)
  }
}

/// Configure a thread-local storage.
///
/// See the [`module-level documentation`] for more details.
///
/// [`module-level documentation`]: thread/index.html
pub macro thread_local($($tokens:tt)*) {
  $crate::thread::thread_local_imp!($($tokens)*);
}
