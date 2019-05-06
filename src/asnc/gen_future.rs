use crate::thr::current_task;
use core::{
  future::Future,
  marker::Unpin,
  ops::{Generator, GeneratorState},
  pin::Pin,
  task::{Context, Poll},
};

/// Wrap a future in a generator.
#[inline]
pub fn asnc<T: Generator<Yield = ()>>(x: T) -> impl Future<Output = T::Return> {
  GenFuture(x)
}

#[must_use]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct GenFuture<T: Generator<Yield = ()>>(T);

impl<T: Generator<Yield = ()>> !Unpin for GenFuture<T> {}

impl<T: Generator<Yield = ()>> Future for GenFuture<T> {
  type Output = T::Return;

  #[inline]
  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    let gen = unsafe { self.map_unchecked_mut(|x| &mut x.0) };
    current_task().set_context(cx, || match gen.resume() {
      GeneratorState::Yielded(()) => Poll::Pending,
      GeneratorState::Complete(x) => Poll::Ready(x),
    })
  }
}
