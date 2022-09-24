use core::marker::Unpin;
use core::pin::Pin;

use crate::fib::{Fiber, FiberState, RootFiber};
use crate::thr::prelude::*;

/// Fiber for [`FnMut`] closure.
///
/// Can be created with [`fib::new_fn`](crate::fib::new_fn).
pub struct FiberFn<F, Y, R>(Option<F>)
where
    F: FnMut() -> FiberState<Y, R>;

/// Fiber for [`FnOnce`] closure.
///
/// Can be created with [`fib::new_once`](crate::fib::new_once).
pub struct FiberOnce<F, R>(Option<F>)
where
    F: FnOnce() -> R,
    F: Unpin;

#[marker]
pub trait ReturnNone: Send + 'static {}

impl ReturnNone for () {}
impl ReturnNone for ! {}

impl<F, Y, R> Fiber for FiberFn<F, Y, R>
where
    F: FnMut() -> FiberState<Y, R>,
{
    type Input = ();
    type Return = R;
    type Yield = Y;

    fn resume(self: Pin<&mut Self>, (): ()) -> FiberState<Y, R> {
        let option = unsafe { &mut self.get_unchecked_mut().0 };
        match option {
            Some(f) => {
                let state = f();
                if state.is_complete() {
                    *option = None;
                }
                state
            }
            None => panic!("fiber resumed after completion"),
        }
    }
}

#[allow(clippy::mismatching_type_param_order)]
impl<F, R> RootFiber for FiberFn<F, (), R>
where
    F: FnMut() -> FiberState<(), R>,
    F: 'static,
    R: ReturnNone,
{
    #[inline]
    fn advance(self: Pin<&mut Self>) -> bool {
        match self.resume(()) {
            FiberState::Yielded(()) => false,
            FiberState::Complete(_) => true,
        }
    }
}

impl<F, R> Fiber for FiberOnce<F, R>
where
    F: FnOnce() -> R,
    F: Unpin,
{
    type Input = ();
    type Return = R;
    type Yield = !;

    fn resume(self: Pin<&mut Self>, (): ()) -> FiberState<!, R> {
        if let Some(f) = self.get_mut().0.take() {
            FiberState::Complete(f())
        } else {
            panic!("fiber resumed after completion");
        }
    }
}

impl<F> RootFiber for FiberOnce<F, ()>
where
    F: FnOnce(),
    F: Unpin + 'static,
{
    #[inline]
    fn advance(self: Pin<&mut Self>) -> bool {
        match self.resume(()) {
            FiberState::Complete(()) => true,
        }
    }
}

/// Creates a fiber that runs the closure `f` until [`FiberState::Complete`] is
/// returned.
#[inline]
pub fn new_fn<F, Y, R>(f: F) -> FiberFn<F, Y, R>
where
    F: FnMut() -> FiberState<Y, R>,
{
    FiberFn(Some(f))
}

/// Creates a fiber that calls the closure `f` once.
///
/// This type of fiber will never yield and will busy its thread until
/// completion.
#[inline]
pub fn new_once<F, R>(f: F) -> FiberOnce<F, R>
where
    F: FnOnce() -> R,
    F: Unpin,
{
    FiberOnce(Some(f))
}

/// Extends [`ThrToken`](crate::thr::ThrToken) types with `add_fn`,
/// `add_fn_factory`, and `add_once` methods.
pub trait ThrFiberClosure: ThrToken {
    /// Adds a fiber that runs the closure `f` until [`FiberState::Complete`] is
    /// returned.
    #[inline]
    fn add_fn<F, R>(self, f: F)
    where
        F: FnMut() -> FiberState<(), R>,
        F: Send + 'static,
        R: ReturnNone,
    {
        self.add_fib(new_fn(f));
    }

    /// Adds a fiber that runs the closure returned by `factory` until
    /// [`FiberState::Complete`] is returned.
    ///
    /// This method is useful for non-`Send` fibers.
    #[inline]
    fn add_fn_factory<C, F, R>(self, factory: C)
    where
        C: FnOnce() -> F + Send + 'static,
        F: FnMut() -> FiberState<(), R>,
        F: 'static,
        R: ReturnNone,
    {
        self.add_fib_factory(|| new_fn(factory()));
    }

    /// Adds a fiber that calls the closure `f` once.
    #[inline]
    fn add_once<F>(self, f: F)
    where
        F: FnOnce(),
        F: Unpin + Send + 'static,
    {
        self.add_fib(new_once(f));
    }
}

impl<T: ThrToken> ThrFiberClosure for T {}
