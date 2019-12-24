use crate::{
    fib::{Fiber, FiberRoot, FiberState},
    thr::prelude::*,
};
use core::{marker::Unpin, pin::Pin};

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

impl<F, R> FiberRoot for FiberFn<F, (), R>
where
    F: FnMut() -> FiberState<(), R>,
    F: Send + 'static,
    R: ReturnNone,
{
    #[inline]
    fn advance(self: Pin<&mut Self>) -> bool {
        match self.resume(()) {
            FiberState::Yielded(()) => true,
            FiberState::Complete(_) => false,
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

impl<F> FiberRoot for FiberOnce<F, ()>
where
    F: FnOnce(),
    F: Unpin + Send + 'static,
{
    #[inline]
    fn advance(self: Pin<&mut Self>) -> bool {
        match self.resume(()) {
            FiberState::Complete(()) => false,
        }
    }
}

/// Creates a fiber from the closure `f`.
#[inline]
pub fn new_fn<F, Y, R>(f: F) -> FiberFn<F, Y, R>
where
    F: FnMut() -> FiberState<Y, R>,
{
    FiberFn(Some(f))
}

/// Creates a fiber from the closure `f`.
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

/// Extends [`ThrToken`](crate::thr::ThrToken) types with `add_fn` and
/// `add_once` methods.
pub trait ThrFiberClosure: ThrToken {
    /// Adds a fiber for the closure `f` to the fiber chain.
    #[inline]
    fn add_fn<F, R>(self, f: F)
    where
        F: FnMut() -> FiberState<(), R>,
        F: Send + 'static,
        R: ReturnNone,
    {
        self.add_fib(new_fn(f))
    }

    /// Adds a fiber for the closure `f` to the fiber chain.
    #[inline]
    fn add_once<F>(self, f: F)
    where
        F: FnOnce(),
        F: Unpin + Send + 'static,
    {
        self.add_fib(new_once(f))
    }
}

impl<T: ThrToken> ThrFiberClosure for T {}
