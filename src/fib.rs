//! The Fibers module.
//!
//! A fiber is a task unit of Drone. It is a stack-less co-routine programmed
//! with async/await, generator, or closure Rust syntaxes. Any number of fibers
//! can be added to a particular thread. A thread executes its fibers in LIFO
//! order. When a fiber yields or completes, the thread proceeds to a next one.

mod chain;
mod closure;
mod future;
mod generator;
mod stream_ring;
mod stream_unit;

pub use self::{
    chain::Chain,
    closure::{new_fn, FiberFn, ThrFiberFn},
    future::{FiberFuture, ThrFiberFuture, TryFiberFuture},
    generator::{new, FiberGen, ThrFiberGen},
    stream_ring::{FiberStreamRing, ThrStreamRing, TryFiberStreamRing},
    stream_unit::{FiberStreamUnit, ThrStreamUnit, TryFiberStreamUnit},
};

use core::pin::Pin;

/// The main task unit of Drone.
pub trait Fiber {
    /// The type of value this fiber consumes on each
    /// [`resume`][`Fiber::resume`].
    type Input;

    /// The type of value this fiber yields.
    type Yield;

    /// The type of value this fiber returns on completion.
    type Return;

    /// Resumes the execution of this fiber.
    ///
    /// This method will resume execution of the fiber or start execution if it
    /// hasn't already.
    ///
    /// # Return value
    ///
    /// The [`FiberState`] enum returned from this method indicates what state
    /// the fiber is in upon returning. If [`FiberState::Yielded`] is returned
    /// then the fiber has reached a suspension point and a value has been
    /// yielded out. Fibers in this state are available for resumption on a
    /// later point.
    ///
    /// If [`FiberState::Complete`] is returned then the fiber has completely
    /// finished with the value provided. It is invalid for the fiber to be
    /// resumed again.
    ///
    /// # Panics
    ///
    /// This method may panic if it is called after [`FiberState::Complete`] has
    /// been returned previously.
    fn resume(self: Pin<&mut Self>, input: Self::Input) -> FiberState<Self::Yield, Self::Return>;
}

/// The root fiber trait.
///
/// A variation of [`Fiber`] with `Input` being `()`, `Yield` - `()` or `!`,
/// `Complete` - `()`.
pub trait FiberRoot: Send + 'static {
    /// Resumes the execution of this fiber.
    ///
    /// This method will resume execution of the fiber or start execution if it
    /// hasn't already.
    ///
    /// # Return value
    ///
    /// If `true` is returned then the fiber has reached a suspension
    /// point. Fibers in this state are available for resumption on a later
    /// point.
    ///
    /// If `false` is returned then the fiber has completely finished. It is
    /// invalid for the fiber to be resumed again.
    ///
    /// # Panics
    ///
    /// This method may panic if it is called after `false` has been returned
    /// previously.
    fn advance(self: Pin<&mut Self>) -> bool;
}

/// The result of a fiber resumption.
///
/// The enum is returned from the [`Fiber::resume`] method and indicates the
/// possible return value of a fiber.
pub enum FiberState<Y, R> {
    /// The fiber suspended with a value.
    Yielded(Y),
    /// The fiber completed with a return value.
    Complete(R),
}
