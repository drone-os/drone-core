//! Interrupt-driven threads.
//!
//! # The threading model
//!
//! A Drone application consists of a static collection of threads. Each thread
//! consists of a dynamic chain of routines, which are executing sequentially
//! within a thread context.

#[doc(hidden)] // FIXME https://github.com/rust-lang/rust/issues/45266
mod chain;
#[doc(hidden)] // FIXME https://github.com/rust-lang/rust/issues/45266
mod executor;
#[doc(hidden)] // FIXME https://github.com/rust-lang/rust/issues/45266
mod future;

pub use self::chain::Chain;
pub use self::executor::Executor;
pub use self::future::ThreadFuture;
pub use drone_macros::thread_local_imp;

use core::ops::Generator;
use futures::{task, Async, Future};

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

/// Initialize the `futures` task system.
///
/// # Safety
///
/// Must be called before using `futures`.
#[inline]
pub unsafe fn init<T>() -> bool
where
  T: Thread + 'static,
{
  task::init(get_task::<T>, set_task::<T>)
}

/// Configure a thread-local storage.
///
/// See the [`module-level documentation`] for more details.
///
/// [`module-level documentation`]: thread/index.html
pub macro thread_local($($tokens:tt)*) {
  $crate::thread::thread_local_imp!($($tokens)*);
}

/// A thread interface.
pub trait Thread: Sized {
  /// Returns a reference to a thread by `id`.
  unsafe fn get_unchecked(id: usize) -> &'static Self;

  /// Provides a reference to the chain of routines.
  fn chain(&self) -> &Chain;

  /// Provides a mutable reference to the chain of routines.
  fn chain_mut(&mut self) -> &mut Chain;

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
  /// Completed routines will be dropped.
  ///
  /// # Safety
  ///
  /// Must not be called concurrently.
  unsafe fn run(&mut self, id: usize) {
    self.set_preempted_id(current_id());
    set_current_id(id);
    self.chain_mut().drain();
    set_current_id(self.preempted_id());
  }

  /// Attaches a new routine to the thread.
  fn routine<G>(&self, g: G)
  where
    G: Generator<Yield = (), Return = ()>,
    G: Send + 'static,
  {
    self.chain().push(g);
  }

  /// Attaches a new closure to the thread.
  fn callback<F>(&self, f: F)
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

  /// Attaches a new future to the thread.
  fn future<G, R, E>(&self, g: G) -> ThreadFuture<R, E>
  where
    G: Generator<Yield = (), Return = Result<R, E>>,
    G: Send + 'static,
    R: Send + 'static,
    E: Send + 'static,
  {
    ThreadFuture::new(self, g)
  }

  /// Attaches a new future executor to the thread.
  fn exec<F>(&self, f: F)
  where
    F: Future<Item = (), Error = ()>,
    F: Send + 'static,
  {
    let mut executor = Executor::new(f);
    self.routine(move || loop {
      match executor.poll() {
        Ok(Async::NotReady) => (),
        Ok(Async::Ready(())) | Err(()) => break,
      }
      yield;
    });
  }
}

#[doc(hidden)] // FIXME https://github.com/rust-lang/rust/issues/45266
fn get_task<T>() -> *mut u8
where
  T: Thread + 'static,
{
  current::<T>().task()
}

#[doc(hidden)] // FIXME https://github.com/rust-lang/rust/issues/45266
#[cfg_attr(feature = "clippy", allow(not_unsafe_ptr_arg_deref))]
fn set_task<T>(task: *mut u8)
where
  T: Thread + 'static,
{
  unsafe { current::<T>().set_task(task) }
}
