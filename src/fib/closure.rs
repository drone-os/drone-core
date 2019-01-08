use crate::{
  fib::{Fiber, FiberRoot, FiberState},
  thr::prelude::*,
};

/// Closure fiber.
pub struct FiberFn<F, R>(Option<F>)
where
  F: FnOnce() -> R;

impl<F, R> Fiber for FiberFn<F, R>
where
  F: FnOnce() -> R,
{
  type Input = ();
  type Yield = !;
  type Return = R;

  fn resume(&mut self, _input: ()) -> FiberState<!, R> {
    FiberState::Complete(match self.0.take() {
      Some(f) => f(),
      None => panic!("closure fiber resumed after completion"),
    })
  }
}

impl<F> FiberRoot for FiberFn<F, ()>
where
  F: FnOnce(),
  F: Send + 'static,
{
  #[inline]
  fn advance(&mut self) -> bool {
    match self.resume(()) {
      FiberState::Complete(()) => false,
    }
  }
}

/// Creates a new closure fiber.
#[inline]
pub fn new_fn<F, R>(f: F) -> FiberFn<F, R>
where
  F: FnOnce() -> R,
{
  FiberFn(Some(f))
}

/// Closure fiber extension to the thread token.
pub trait ThrFiberFn<T: ThrAttach>: ThrToken<T> {
  /// Adds a new closure fiber.
  fn add_fn<F>(self, f: F)
  where
    F: FnOnce(),
    F: Send + 'static,
  {
    self.add_fib(new_fn(f))
  }
}

impl<T: ThrAttach, U: ThrToken<T>> ThrFiberFn<T> for U {}
