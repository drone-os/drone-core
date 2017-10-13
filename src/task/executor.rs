//! A task executor.
//!
//! See [`Executor`] for more details.
//!
//! [`Executor`]: struct.Executor.html

use futures::Async;
use futures::executor::{self, Notify, Spawn};

/// A task executor.
///
/// This `struct` is created by the [`executor`] method on [`DroneFuture`]. See
/// its documentation for more.
///
/// [`DroneFuture`]: ../trait.DroneFuture.html
/// [`executor`]: ../trait.DroneFuture.html#method.executor
pub struct Executor<T>
where
  T: Future<Item = (), Error = ()>,
{
  task: Spawn<T>,
}

impl<T> Executor<T>
where
  T: Future<Item = (), Error = ()>,
{
  pub(crate) fn new(future: T) -> Self {
    let task = executor::spawn(future);
    Self { task }
  }

  /// Runs the executor. Returns `false` if completed, and `true` if not.
  pub fn run(&mut self) -> bool {
    match self.task.poll_future_notify(&&NO_NOTIFY, 0) {
      Ok(Async::NotReady) => true,
      Ok(Async::Ready(())) | Err(()) => false,
    }
  }
}

struct NoNotify;

const NO_NOTIFY: NoNotify = NoNotify;

impl Notify for NoNotify {
  fn notify(&self, _id: usize) {}
}
