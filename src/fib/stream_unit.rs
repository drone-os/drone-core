use crate::{
    fib::{Fiber, FiberState},
    sync::spsc::unit::{channel, Receiver, SendError},
    thr::prelude::*,
};
use core::{
    convert::identity,
    pin::Pin,
    task::{Context, Poll},
};
use futures::Stream;

/// A stream of `()` from the fiber in another thread.
///
/// Dropping or closing this future will remove the fiber on a next thread
/// invocation without resuming it.
#[must_use]
pub struct FiberStreamUnit {
    rx: Receiver<!>,
}

/// A stream of `Result<(), E>` from the fiber in another thread.
///
/// Dropping or closing this future will remove the fiber on a next thread
/// invocation without resuming it.
#[must_use]
pub struct TryFiberStreamUnit<E> {
    rx: Receiver<E>,
}

impl FiberStreamUnit {
    /// Gracefully close this future.
    ///
    /// The fiber will be removed on a next thread invocation without resuming.
    #[inline]
    pub fn close(&mut self) {
        self.rx.close()
    }
}

impl<E> TryFiberStreamUnit<E> {
    /// Gracefully close this future.
    ///
    /// The fiber will be removed on a next thread invocation without resuming.
    #[inline]
    pub fn close(&mut self) {
        self.rx.close()
    }
}

impl Stream for FiberStreamUnit {
    type Item = ();

    #[inline]
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let rx = unsafe { self.map_unchecked_mut(|x| &mut x.rx) };
        rx.poll_next(cx).map(|value| {
            value.map(|value| match value {
                Ok(value) => value,
            })
        })
    }
}

impl<E> Stream for TryFiberStreamUnit<E> {
    type Item = Result<(), E>;

    #[inline]
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let rx = unsafe { self.map_unchecked_mut(|x| &mut x.rx) };
        rx.poll_next(cx)
    }
}

/// Extends [`ThrToken`][`crate::thr::ThrToken`] types with `add_stream`
/// methods.
pub trait ThrStreamUnit: ThrToken {
    /// Adds the fiber `fib` to the fiber chain and returns a stream of `()`
    /// yielded from the fiber.
    fn add_stream_skip<F>(self, fib: F) -> FiberStreamUnit
    where
        F: Fiber<Input = (), Yield = Option<()>, Return = Option<()>>,
        F: Send + 'static,
    {
        FiberStreamUnit {
            rx: add_rx(self, || Ok(()), fib, Ok),
        }
    }

    /// Adds the fiber `fib` to the fiber chain and returns a stream of
    /// `Result<(), E>` yielded from the fiber.
    fn add_stream<O, F, E>(self, overflow: O, fib: F) -> TryFiberStreamUnit<E>
    where
        O: Fn() -> Result<(), E>,
        F: Fiber<Input = (), Yield = Option<()>, Return = Result<Option<()>, E>>,
        O: Send + 'static,
        F: Send + 'static,
        E: Send + 'static,
    {
        TryFiberStreamUnit {
            rx: add_rx(self, overflow, fib, identity),
        }
    }
}

#[inline]
fn add_rx<T, O, F, E, C>(thr: T, overflow: O, mut fib: F, convert: C) -> Receiver<E>
where
    T: ThrToken,
    O: Fn() -> Result<(), E>,
    F: Fiber<Input = (), Yield = Option<()>>,
    C: FnOnce(F::Return) -> Result<Option<()>, E>,
    O: Send + 'static,
    F: Send + 'static,
    E: Send + 'static,
    C: Send + 'static,
{
    let (rx, mut tx) = channel();
    thr.add(move || {
        loop {
            if tx.is_canceled() {
                break;
            }
            match unsafe { Pin::new_unchecked(&mut fib) }.resume(()) {
                FiberState::Yielded(None) => {}
                FiberState::Yielded(Some(())) => match tx.send() {
                    Ok(()) => {}
                    Err(SendError::Canceled) => {
                        break;
                    }
                    Err(SendError::Overflow) => match overflow() {
                        Ok(()) => {}
                        Err(err) => {
                            tx.send_err(err).ok();
                            break;
                        }
                    },
                },
                FiberState::Complete(value) => {
                    match convert(value) {
                        Ok(None) => {}
                        Ok(Some(())) => {
                            tx.send().ok();
                        }
                        Err(err) => {
                            tx.send_err(err).ok();
                        }
                    }
                    break;
                }
            }
            yield;
        }
    });
    rx
}

impl<T: ThrToken> ThrStreamUnit for T {}
