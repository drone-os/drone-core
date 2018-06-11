use core::ops::{Generator, GeneratorState};
use futures::prelude::*;
use thr::__current_task;

#[must_use]
pub struct GenFuture<R, E, G>(G)
where
  G: Generator<Yield = (), Return = Result<R, E>>;

impl<R, E, G> GenFuture<R, E, G> where
  G: Generator<Yield = (), Return = Result<R, E>>
{}

/// Creates a new generator-based future.
#[inline(always)]
pub fn async<R, E, G>(gen: G) -> GenFuture<R, E, G>
where
  G: Generator<Yield = (), Return = Result<R, E>>,
{
  GenFuture(gen)
}

impl<R, E, G> Future for GenFuture<R, E, G>
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
