use crate::{
    fib::{Fiber, FiberState},
    sync::spsc::oneshot::{channel, Receiver, RecvError},
    thr::prelude::*,
};
use core::{
    convert::identity,
    future::Future,
    intrinsics::unreachable,
    pin::Pin,
    task::{Context, Poll},
};

/// A future that resolves on completion of the fiber from another thread.
///
/// Dropping or closing this future will remove the fiber on a next thread
/// invocation without resuming it.
#[must_use]
pub struct FiberFuture<R> {
    rx: Receiver<R, !>,
}

/// A future that resolves on completion of the fallible fiber from another
/// thread.
///
/// Dropping or closing this future will remove the fiber on a next thread
/// invocation without resuming it.
#[must_use]
pub struct TryFiberFuture<R, E> {
    rx: Receiver<R, E>,
}

#[marker]
pub trait YieldNone: Send + 'static {}

impl YieldNone for () {}
impl YieldNone for ! {}

impl<R> FiberFuture<R> {
    /// Gracefully close this future.
    ///
    /// The fiber will be removed on a next thread invocation without resuming.
    #[inline]
    pub fn close(&mut self) {
        self.rx.close()
    }
}

impl<R, E> TryFiberFuture<R, E> {
    /// Gracefully close this future.
    ///
    /// The fiber will be removed on a next thread invocation without resuming.
    #[inline]
    pub fn close(&mut self) {
        self.rx.close()
    }
}

impl<R> Future for FiberFuture<R> {
    type Output = R;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<R> {
        let rx = unsafe { self.map_unchecked_mut(|x| &mut x.rx) };
        rx.poll(cx).map(|value| match value {
            Ok(value) => value,
            Err(RecvError::Canceled) => unsafe { unreachable() },
        })
    }
}

impl<R, E> Future for TryFiberFuture<R, E> {
    type Output = Result<R, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<R, E>> {
        let rx = unsafe { self.map_unchecked_mut(|x| &mut x.rx) };
        rx.poll(cx).map_err(|err| match err {
            RecvError::Complete(err) => err,
            RecvError::Canceled => unsafe { unreachable() },
        })
    }
}

/// Extends [`ThrToken`][`crate::thr::ThrToken`] types with `add_future`
/// methods.
pub trait ThrFiberFuture: ThrToken {
    /// Adds the fiber `fib` to the fiber chain and returns a future, which
    /// resolves on completion of the fiber.
    fn add_future<F, Y, R>(self, fib: F) -> FiberFuture<R>
    where
        F: Fiber<Input = (), Yield = Y, Return = R>,
        Y: YieldNone,
        F: Send + 'static,
        R: Send + 'static,
    {
        FiberFuture {
            rx: add_rx(self, fib, Ok),
        }
    }

    /// Adds the fallible fiber `fib` to the fiber chain and returns a future,
    /// which resolves on completion of the fiber.
    fn add_try_future<F, Y, R, E>(self, fib: F) -> TryFiberFuture<R, E>
    where
        F: Fiber<Input = (), Yield = Y, Return = Result<R, E>>,
        Y: YieldNone,
        F: Send + 'static,
        R: Send + 'static,
        E: Send + 'static,
    {
        TryFiberFuture {
            rx: add_rx(self, fib, identity),
        }
    }
}

#[inline]
fn add_rx<T, F, Y, R, E, C>(thr: T, mut fib: F, convert: C) -> Receiver<R, E>
where
    T: ThrToken,
    F: Fiber<Input = (), Yield = Y>,
    Y: YieldNone,
    C: FnOnce(F::Return) -> Result<R, E>,
    F: Send + 'static,
    R: Send + 'static,
    E: Send + 'static,
    C: Send + 'static,
{
    let (rx, tx) = channel();
    thr.add(move || {
        loop {
            if tx.is_canceled() {
                break;
            }
            match unsafe { Pin::new_unchecked(&mut fib) }.resume(()) {
                FiberState::Yielded(_) => {}
                FiberState::Complete(complete) => {
                    tx.send(convert(complete)).ok();
                    break;
                }
            }
            yield;
        }
    });
    rx
}

impl<T: ThrToken> ThrFiberFuture for T {}
