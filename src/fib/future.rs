use crate::{
    fib::{Fiber, FiberState},
    sync::spsc::oneshot::{channel, Canceled, Receiver},
    thr::prelude::*,
};
use core::{
    future::Future,
    intrinsics::unreachable,
    pin::Pin,
    task::{Context, Poll},
};

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
        self.rx.close()
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

/// Extends [`ThrToken`](crate::thr::ThrToken) types with `add_future` method.
pub trait ThrFiberFuture: ThrToken {
    /// Adds the fiber `fib` to the fiber chain and returns a future, which
    /// resolves on completion of the fiber.
    fn add_future<F, Y, T>(self, fib: F) -> FiberFuture<T>
    where
        F: Fiber<Input = (), Yield = Y, Return = T>,
        Y: YieldNone,
        F: Send + 'static,
        T: Send + 'static,
    {
        FiberFuture {
            rx: add_rx(self, fib),
        }
    }
}

#[inline]
fn add_rx<H, F, Y, T>(thr: H, mut fib: F) -> Receiver<T>
where
    H: ThrToken,
    F: Fiber<Input = (), Yield = Y, Return = T>,
    Y: YieldNone,
    F: Send + 'static,
    T: Send + 'static,
{
    let (tx, rx) = channel();
    thr.add(move || {
        loop {
            if tx.is_canceled() {
                break;
            }
            match unsafe { Pin::new_unchecked(&mut fib) }.resume(()) {
                FiberState::Yielded(_) => {}
                FiberState::Complete(complete) => {
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
