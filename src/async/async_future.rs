use core::ops::{Generator, GeneratorState};
use futures::prelude::*;
use thr::__current_task;

/// A generator-based future.
#[must_use]
pub struct AsyncFuture<R, E, G>(G)
where
  G: Generator<Yield = (), Return = Result<R, E>>;

impl<R, E, G> AsyncFuture<R, E, G>
where
  G: Generator<Yield = (), Return = Result<R, E>>,
{
  /// Creates a new `AsyncFuture`.
  #[inline(always)]
  pub fn new(gen: G) -> Self {
    AsyncFuture(gen)
  }
}

impl<R, E, G> Future for AsyncFuture<R, E, G>
where
  G: Generator<Yield = (), Return = Result<R, E>>,
{
  type Item = R;
  type Error = E;

  // FIXME Use `Pin` when implemented
  #[inline(always)]
  fn poll(&mut self, cx: &mut task::Context) -> Poll<R, E> {
    __current_task().__set_cx(cx, || match unsafe { self.0.resume() } {
      GeneratorState::Yielded(()) => Ok(Async::Pending),
      GeneratorState::Complete(complete) => complete.map(Async::Ready),
    })
  }
}
