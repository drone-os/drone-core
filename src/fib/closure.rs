use fib::{Fiber, FiberRoot, FiberState};
use thr::Thread;

/// A closure fiber.
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
  #[inline(always)]
  fn advance(&mut self) -> bool {
    match self.resume(()) {
      FiberState::Complete(()) => false,
    }
  }
}

/// Creates a new closure fiber.
#[inline(always)]
pub fn new_fn<F, R>(f: F) -> FiberFn<F, R>
where
  F: FnOnce() -> R,
{
  FiberFn(Some(f))
}

/// Spawns a new closure fiber on the given `thr`.
#[inline(always)]
pub fn spawn_fn<T, U, F>(thr: T, f: F)
where
  T: AsRef<U>,
  U: Thread,
  F: FnOnce(),
  F: Send + 'static,
{
  thr.as_ref().fib_chain().add(new_fn(f))
}
