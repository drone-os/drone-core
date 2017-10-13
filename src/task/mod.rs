//! Asynchronous tasks.

pub mod drone_future;
pub mod thread_future;
pub mod executor;

pub use self::drone_future::DroneFuture;
pub use self::executor::Executor;
pub use self::thread_future::ThreadFuture;

use futures;
use thread;

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
  futures::task::init(get::<T>, set::<T>)
}

fn get<T>() -> *mut u8
where
  T: Thread + 'static,
{
  thread::current::<T>().task()
}

fn set<T>(task: *mut u8)
where
  T: Thread + 'static,
{
  unsafe { thread::current::<T>().set_task(task) }
}
