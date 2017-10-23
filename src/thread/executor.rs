use futures::{Future, Poll};
use futures::executor::{self, Notify, Spawn};

/// Executor of `Future`.
///
/// Wraps a future to correctly set task environment.
#[must_use]
pub struct Executor<T>
where
  T: Future,
{
  future: Spawn<T>,
}

impl<T> Executor<T>
where
  T: Future,
{
  /// Creates a new `Executor` for the given `future`.
  pub fn new(future: T) -> Self {
    let future = executor::spawn(future);
    Self { future }
  }

  /// Query the inner future to see if its value has become available.
  pub fn poll(&mut self) -> Poll<T::Item, T::Error> {
    self.future.poll_future_notify(&&NOP_NOTIFY, 0)
  }
}

struct NopNotify;

const NOP_NOTIFY: NopNotify = NopNotify;

impl Notify for NopNotify {
  fn notify(&self, _id: usize) {}
}
