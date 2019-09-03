//! The Fibers module.
//!
//! **NOTE** A Drone platform crate may re-export this module with its own
//! additions under the same name, in which case it should be used instead.
//!
//! A fiber is a task unit of Drone. It is a stack-less co-routine programmed
//! with async/await, generator, or closure Rust syntaxes. Any number of fibers
//! can be added to a particular thread. A thread executes its fibers in LIFO
//! order. When a fiber yields, the thread keeps it for the next time it resumes
//! and proceeds to the next fiber. When a fiber returns, the thread drops it
//! and proceeds to the next fiber. When there are no fibers left, the thread
//! suspends.
//!
//! # Basic Fibers
//!
//! A basic fiber can be created with [`fib::new`] or [`fib::new_fn`]:
//!
//! ```
//! # #![feature(generators)]
//! use drone_core::fib;
//!
//! // This is `impl Fiber<Input = (), Yield = i32, Return = i32>`
//! let a = fib::new(|| {
//!     // do some work and yield
//!     yield 1;
//!     // do some work and yield
//!     yield 2;
//!     // do some work and return
//!     3
//! });
//!
//! // Use `fib::new_fn` when there are no `yield`s.
//! // This is `impl Fiber<Input = (), Yield = !, Return = i32>`
//! let b = fib::new_fn(|| {
//!     // do some work and immediately return
//!     4
//! });
//! ```
//!
//! A basic fiber can be attached to a thread with
//! [`token.add(...)`](fib::ThrFiberGen::add) or
//! [`token.add_fn(...)`](fib::ThrFiberFn::add_fn). Note that fibers that are
//! directly attached to threads can't have yield and return values other than
//! `()`.
//!
//! ```
//! # #![feature(generators)]
//! # use drone_core::token::Token;
//! # static mut THREADS: [Thr; 1] = [Thr::new(0)];
//! # drone_core::thr!(use THREADS; struct Thr {} struct ThrLocal {});
//! # #[derive(Clone, Copy)] struct SysTick;
//! # struct Thrs { sys_tick: SysTick }
//! # unsafe impl Token for Thrs {
//! #     unsafe fn take() -> Self { Self { sys_tick: SysTick::take() } }
//! # }
//! # unsafe impl Token for SysTick {
//! #     unsafe fn take() -> Self { Self }
//! # }
//! # unsafe impl drone_core::thr::ThrToken for SysTick {
//! #     type Thr = Thr;
//! #     const THR_NUM: usize = 0;
//! # }
//! # fn main() {
//! #     let thr = unsafe { Thrs::take() };
//! use drone_core::thr::prelude::*;
//!
//! // This is `impl Fiber<Input = (), Yield = (), Return = ()>`
//! thr.sys_tick.add(|| {
//!     // do some work and yield
//!     yield;
//!     // do some work and yield
//!     yield;
//!     // do some work and return
//! });
//!
//! // Use `add_fn` when there are no `yield`s.
//! // This is `impl Fiber<Input = (), Yield = !, Return = ()>`
//! thr.sys_tick.add_fn(|| {
//!     // do some work and immediately return
//! });
//! # }
//! ```
//!
//! # Compound Fibers
//!
//! There is a number of useful compound fibers implemented in this module:
//!
//! | Method                                                                                               | Input / Output                                                                |
//! |------------------------------------------------------------------------------------------------------|-------------------------------------------------------------------------------|
//! | [`token.add_future(...)`](fib::ThrFiberFuture::add_future)                                           | `Fiber<Input = (), Yield = ()/!, Return = T>`                                 |
//! | `->`                                                                                                 | `Future<Output = T>`                                                          |
//! | [`token.add_stream_pulse(...)`](fib::ThrFiberStreamPulse::add_stream_pulse)                          | `Fiber<Input = (), Yield = Option<usize>, Return = Result<Option<usize>, E>>` |
//! | `->`                                                                                                 | `Stream<Item = Result<NonZeroUsize, E>>`                                      |
//! | [`token.add_stream_pulse_skip(...)`](fib::ThrFiberStreamPulse::add_stream_pulse_skip)                | `Fiber<Input = (), Yield = Option<usize>, Return = Option<usize>>`            |
//! | `->`                                                                                                 | `Stream<Item = NonZeroUsize>`                                                 |
//! | [`token.add_stream_ring(...)`](fib::ThrFiberStreamRing::add_stream_ring)                             | `Fiber<Input = (), Yield = Option<T>, Return = Result<Option<T>, E>>`         |
//! | `->`                                                                                                 | `Stream<Item = Result<T, E>>`                                                 |
//! | [`token.add_stream_ring_skip(...)`](fib::ThrFiberStreamRing::add_stream_ring_skip)                   | `Fiber<Input = (), Yield = Option<T>, Return = Option<T>>`                    |
//! | `->`                                                                                                 | `Stream<Item = T>`                                                            |
//! | [`token.add_stream_ring_overwrite(...)`](fib::ThrFiberStreamRing::add_stream_ring_overwrite)         | `Fiber<Input = (), Yield = Option<T>, Return = Option<T>>`                    |
//! | `->`                                                                                                 | `Stream<Item = T>`                                                            |
//! | [`token.add_try_stream_ring_overwrite(...)`](fib::ThrFiberStreamRing::add_try_stream_ring_overwrite) | `Fiber<Input = (), Yield = Option<T>, Return = Result<Option<T>, E>>`         |
//! | `->`                                                                                                 | `Stream<Item = Result<T, E>>`                                                 |
//!
//! ## Examples
//!
//! ```
//! # #![feature(generators)]
//! # use drone_core::token::Token;
//! # static mut THREADS: [Thr; 1] = [Thr::new(0)];
//! # drone_core::thr!(use THREADS; struct Thr {} struct ThrLocal {});
//! # #[derive(Clone, Copy)] struct SysTick;
//! # struct Thrs { sys_tick: SysTick }
//! # unsafe impl Token for Thrs {
//! #     unsafe fn take() -> Self { Self { sys_tick: SysTick::take() } }
//! # }
//! # unsafe impl Token for SysTick {
//! #     unsafe fn take() -> Self { Self }
//! # }
//! # unsafe impl drone_core::thr::ThrToken for SysTick {
//! #     type Thr = Thr;
//! #     const THR_NUM: usize = 0;
//! # }
//! # fn main() {
//! #     let thr = unsafe { Thrs::take() };
//! #     async {
//! use drone_core::{fib, thr::prelude::*};
//!
//! let a = thr.sys_tick.add_future(fib::new(|| {
//!     yield;
//!     yield;
//!     123
//! }));
//!
//! // `b` will have the value of 123 after the SYS_TICK thread has triggered 3
//! // times.
//! let b = a.await;
//! #     };
//! # }
//! ```

mod chain;
mod closure;
mod future;
mod generator;
mod stream_pulse;
mod stream_ring;

pub use self::{
    chain::Chain,
    closure::{new_fn, FiberFn, ThrFiberFn},
    future::{FiberFuture, ThrFiberFuture},
    generator::{new, FiberGen, ThrFiberGen},
    stream_pulse::{FiberStreamPulse, ThrFiberStreamPulse, TryFiberStreamPulse},
    stream_ring::{FiberStreamRing, ThrFiberStreamRing, TryFiberStreamRing},
};

use core::pin::Pin;

/// The main task unit of Drone.
pub trait Fiber {
    /// The type of value this fiber consumes on each [`resume`](Fiber::resume).
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
