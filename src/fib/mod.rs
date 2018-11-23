//! Fibers.

mod chain;
mod closure;
mod future;
mod generator;
mod stream_ring;
mod stream_unit;

pub use self::chain::Chain;
pub use self::closure::{new_fn, FiberFn, ThrFiberFn};
pub use self::future::{FiberFuture, ThrFiberFuture};
pub use self::generator::{new, FiberGen, ThrFiberGen};
pub use self::stream_ring::{FiberStreamRing, ThrStreamRing};
pub use self::stream_unit::{FiberStreamUnit, ThrStreamUnit};

/// Lightweight thread of execution.
pub trait Fiber {
  /// The type of [`resume`](Fiber::resume) input argument.
  type Input;

  /// The type of value this fiber yields.
  type Yield;

  /// The type of value this fiber returns.
  type Return;

  /// Resumes the execution of this fiber.
  fn resume(
    &mut self,
    input: Self::Input,
  ) -> FiberState<Self::Yield, Self::Return>;
}

/// A fiber suitable for [`Chain`](Chain).
pub trait FiberRoot: Send + 'static {
  /// Resumes the execution of this fiber. Returns `true` if it's still alive.
  fn advance(&mut self) -> bool;
}

/// The result of a fiber resumption.
pub enum FiberState<Y, R> {
  /// The fiber suspended with a value.
  Yielded(Y),
  /// The fiber completed with a return value.
  Complete(R),
}

/// One of `()` or `!`.
#[marker]
pub trait YieldNone: Send + 'static {}

impl YieldNone for () {}
impl YieldNone for ! {}
