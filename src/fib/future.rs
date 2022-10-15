use crate::fib;
use crate::fib::Fiber;
use crate::sync::spsc::oneshot::{channel, Canceled, Receiver};
use crate::thr::prelude::*;
use core::future::Future;
use core::intrinsics::unreachable;
use core::pin::Pin;
use core::task::{Context, Poll};

/// A future that resolves on completion of the fiber from another thread.
///
/// Dropping or closing this future will remove the fiber on a next thread
/// invocation without resuming it.
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct FiberFuture<T> {
    rx: Receiver<T>,
}

#[marker]
pub trait YieldNone: Send + 'static {}

impl YieldNone for () {}
impl YieldNone for ! {}

impl<T> FiberFuture<T> {
    /// Gracefully close this future.
    ///
    /// The fiber will be removed on a next thread invocation without resuming.
    #[inline]
    pub fn close(&mut self) {
        self.rx.close();
    }
}

impl<T> Future for FiberFuture<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
        let rx = unsafe { self.map_unchecked_mut(|x| &mut x.rx) };
        rx.poll(cx).map(|value| match value {
            Ok(value) => value,
            Err(Canceled) => unsafe { unreachable() },
        })
    }
}

/// Extends [`ThrToken`](crate::thr::ThrToken) types with `add_future` and
/// `add_future_factory` methods.
pub trait ThrFiberFuture: ThrToken {
    /// Adds the fiber `fib` to the fiber chain and returns a future, which
    /// resolves on fiber completion.
    #[inline]
    fn add_future<F, Y, T>(self, fib: F) -> FiberFuture<T>
    where
        F: Fiber<Input = (), Yield = Y, Return = T>,
        Y: YieldNone,
        F: Send + 'static,
        T: Send + 'static,
    {
        FiberFuture { rx: add_rx(self, || fib) }
    }

    /// Adds the fiber returned by `factory` to the fiber chain and returns a
    /// future, which resolves on fiber completion.
    ///
    /// This method is useful for non-`Send` fibers.
    #[inline]
    fn add_future_factory<C, F, Y, T>(self, factory: C) -> FiberFuture<T>
    where
        C: FnOnce() -> F + Send + 'static,
        F: Fiber<Input = (), Yield = Y, Return = T>,
        Y: YieldNone,
        F: 'static,
        T: Send + 'static,
    {
        FiberFuture { rx: add_rx(self, factory) }
    }
}

#[inline]
fn add_rx<C, H, F, Y, T>(thr: H, factory: C) -> Receiver<T>
where
    C: FnOnce() -> F + Send + 'static,
    H: ThrToken,
    F: Fiber<Input = (), Yield = Y, Return = T>,
    Y: YieldNone,
    F: 'static,
    T: Send + 'static,
{
    let (tx, rx) = channel();
    thr.add_factory(|| {
        let mut fib = factory();
        move || loop {
            if tx.is_canceled() {
                break;
            }
            match unsafe { Pin::new_unchecked(&mut fib) }.resume(()) {
                fib::Yielded(_) => {}
                fib::Complete(complete) => {
                    drop(tx.send(complete));
                    break;
                }
            }
            yield;
        }
    });
    rx
}

impl<T: ThrToken> ThrFiberFuture for T {}
