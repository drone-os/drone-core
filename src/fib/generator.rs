use crate::{
  fib::{Fiber, FiberRoot, FiberState},
  thr::prelude::*,
};
use core::{
  ops::{Generator, GeneratorState},
  pin::Pin,
};

/// Generator fiber.
pub struct FiberGen<G>(G)
where
  G: Generator;

impl<G> Fiber for FiberGen<G>
where
  G: Generator,
{
  type Input = ();
  type Yield = G::Yield;
  type Return = G::Return;

  #[inline]
  fn resume(
    self: Pin<&mut Self>,
    _input: (),
  ) -> FiberState<G::Yield, G::Return> {
    let gen = unsafe { self.map_unchecked_mut(|x| &mut x.0) };
    gen.resume().into()
  }
}

impl<G> FiberRoot for FiberGen<G>
where
  G: Generator<Yield = (), Return = ()>,
  G: Send + 'static,
{
  #[inline]
  fn advance(self: Pin<&mut Self>) -> bool {
    match self.resume(()) {
      FiberState::Yielded(()) => true,
      FiberState::Complete(()) => false,
    }
  }
}

impl<Y, R> From<GeneratorState<Y, R>> for FiberState<Y, R> {
  #[inline]
  fn from(state: GeneratorState<Y, R>) -> Self {
    match state {
      GeneratorState::Yielded(val) => FiberState::Yielded(val),
      GeneratorState::Complete(val) => FiberState::Complete(val),
    }
  }
}

/// Creates a new generator fiber.
#[inline]
pub fn new<G>(gen: G) -> FiberGen<G>
where
  G: Generator,
{
  FiberGen(gen)
}

/// Generator fiber extension to the thread token.
pub trait ThrFiberGen<T: ThrAttach>: ThrToken<T> {
  /// Adds a new generator fiber.
  fn add<G>(self, gen: G)
  where
    G: Generator<Yield = (), Return = ()>,
    G: Send + 'static,
  {
    self.add_fib(new(gen))
  }
}

impl<T: ThrAttach, U: ThrToken<T>> ThrFiberGen<T> for U {}
