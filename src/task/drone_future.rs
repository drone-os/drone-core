//! Drone extension for `Future`.
//!
//! See [`DroneFuture`] for more details.
//!
//! [`DroneFuture`]: trait.DroneFuture.html

use task::Executor;

/// Drone extension for `Future`.
pub trait DroneFuture: Future {
  /// Spawns the `Future` on the `thread`.
  fn spawn<T>(self, thread: &T)
  where
    Self: Future<Item = (), Error = ()>,
    Self: Sized + Send + 'static,
    T: Thread,
  {
    let mut executor = self.executor();
    thread.spawn(move || while executor.run() {
      yield
    });
  }

  /// Returns a task executor for the future.
  fn executor(self) -> Executor<Self>
  where
    Self: Future<Item = (), Error = ()>,
    Self: Sized,
  {
    Executor::new(self)
  }
}

impl<T> DroneFuture for T
where
  T: Future,
{
}
