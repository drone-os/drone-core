use crate::{
    fib::{Fiber, FiberRoot, FiberState},
    thr::prelude::*,
};
use core::{
    ops::{Generator, GeneratorState},
    pin::Pin,
};

/// Fiber for [`Generator`].
///
/// Can be created with [`fib::new`](crate::fib::new).
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
    fn resume(self: Pin<&mut Self>, (): ()) -> FiberState<G::Yield, G::Return> {
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

/// Creates a fiber from the generator `gen`.
///
/// This type of fiber yields on each generator `yield`.
#[inline]
pub fn new<G>(gen: G) -> FiberGen<G>
where
    G: Generator,
{
    FiberGen(gen)
}

/// Extends [`ThrToken`](crate::thr::ThrToken) types with `add` method.
pub trait ThrFiberGen: ThrToken {
    /// Adds a fiber for the generator `gen` to the fiber chain.
    #[inline]
    fn add<G>(self, gen: G)
    where
        G: Generator<Yield = (), Return = ()>,
        G: Send + 'static,
    {
        self.add_fib(new(gen))
    }
}

impl<T: ThrToken> ThrFiberGen for T {}
