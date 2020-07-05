use crate::{
    fib::{self, Fiber},
    sync::spsc::ring::{channel, Receiver, SendError, SendErrorKind},
    thr::prelude::*,
};
use core::{
    convert::identity,
    pin::Pin,
    task::{Context, Poll},
};
use futures::Stream;

/// A stream of `T` from the fiber in another thread.
///
/// Dropping or closing this future will remove the fiber on a next thread
/// invocation without resuming it.
#[must_use = "streams do nothing unless you `.await` or poll them"]
pub struct FiberStreamRing<T> {
    rx: Receiver<T, !>,
}

/// A stream of `Result<T, E>` from the fiber in another thread.
///
/// Dropping or closing this future will remove the fiber on a next thread
/// invocation without resuming it.
#[must_use = "streams do nothing unless you `.await` or poll them"]
pub struct TryFiberStreamRing<T, E> {
    rx: Receiver<T, E>,
}

impl<T> FiberStreamRing<T> {
    /// Gracefully close this future.
    ///
    /// The fiber will be removed on a next thread invocation without resuming.
    #[inline]
    pub fn close(&mut self) {
        self.rx.close()
    }
}

impl<T, E> TryFiberStreamRing<T, E> {
    /// Gracefully close this future.
    ///
    /// The fiber will be removed on a next thread invocation without resuming.
    #[inline]
    pub fn close(&mut self) {
        self.rx.close()
    }
}

impl<T> Stream for FiberStreamRing<T> {
    type Item = T;

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

impl<T, E> Stream for TryFiberStreamRing<T, E> {
    type Item = Result<T, E>;

    #[inline]
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let rx = unsafe { self.map_unchecked_mut(|x| &mut x.rx) };
        rx.poll_next(cx)
    }
}

/// Extends [`ThrToken`](crate::thr::ThrToken) types with ring stream methods.
pub trait ThrFiberStreamRing: ThrToken {
    /// Adds the fiber `fib` to the fiber chain and returns a stream of `T`
    /// yielded from the fiber.
    ///
    /// When the underlying ring buffer overflows, new items will be skipped.
    #[inline]
    fn add_saturating_stream<F, T>(self, capacity: usize, fib: F) -> FiberStreamRing<T>
    where
        F: Fiber<Input = (), Yield = Option<T>, Return = Option<T>>,
        F: Send + 'static,
        T: Send + 'static,
    {
        FiberStreamRing { rx: add_rx(self, capacity, |_| Ok(()), fib, Ok) }
    }

    /// Adds the fiber `fib` to the fiber chain and returns a stream of `T`
    /// yielded from the fiber.
    ///
    /// When the underlying ring buffer overflows, new items will overwrite
    /// existing ones.
    #[inline]
    fn add_overwriting_stream<F, T>(self, capacity: usize, fib: F) -> FiberStreamRing<T>
    where
        F: Fiber<Input = (), Yield = Option<T>, Return = Option<T>>,
        F: Send + 'static,
        T: Send + 'static,
    {
        FiberStreamRing { rx: add_rx_overwrite(self, capacity, fib, Ok) }
    }

    /// Adds the fiber `fib` to the fiber chain and returns a stream of
    /// `Result<T, E>` yielded from the fiber.
    ///
    /// When the underlying ring buffer overflows, new items will be skipped.
    #[inline]
    fn add_try_stream<O, F, T, E>(
        self,
        capacity: usize,
        overflow: O,
        fib: F,
    ) -> TryFiberStreamRing<T, E>
    where
        O: Fn(T) -> Result<(), E>,
        F: Fiber<Input = (), Yield = Option<T>, Return = Result<Option<T>, E>>,
        O: Send + 'static,
        F: Send + 'static,
        T: Send + 'static,
        E: Send + 'static,
    {
        TryFiberStreamRing { rx: add_rx(self, capacity, overflow, fib, identity) }
    }

    /// Adds the fiber `fib` to the fiber chain and returns a stream of
    /// `Result<T, E>` yielded from the fiber.
    ///
    /// When the underlying ring buffer overflows, new items will overwrite
    /// existing ones.
    #[inline]
    fn add_overwriting_try_stream<F, T, E>(
        self,
        capacity: usize,
        fib: F,
    ) -> TryFiberStreamRing<T, E>
    where
        F: Fiber<Input = (), Yield = Option<T>, Return = Result<Option<T>, E>>,
        F: Send + 'static,
        T: Send + 'static,
        E: Send + 'static,
    {
        TryFiberStreamRing { rx: add_rx_overwrite(self, capacity, fib, identity) }
    }
}

#[inline]
fn add_rx<H, O, F, T, E, C>(
    thr: H,
    capacity: usize,
    overflow: O,
    mut fib: F,
    convert: C,
) -> Receiver<T, E>
where
    H: ThrToken,
    O: Fn(T) -> Result<(), E>,
    F: Fiber<Input = (), Yield = Option<T>>,
    C: FnOnce(F::Return) -> Result<Option<T>, E>,
    O: Send + 'static,
    F: Send + 'static,
    T: Send + 'static,
    E: Send + 'static,
    C: Send + 'static,
{
    let (mut tx, rx) = channel(capacity);
    thr.add(move || {
        loop {
            if tx.is_canceled() {
                break;
            }
            match unsafe { Pin::new_unchecked(&mut fib) }.resume(()) {
                fib::Yielded(None) => {}
                fib::Yielded(Some(value)) => match tx.send(value) {
                    Ok(()) => {}
                    Err(SendError { value, kind }) => match kind {
                        SendErrorKind::Canceled => {
                            break;
                        }
                        SendErrorKind::Overflow => match overflow(value) {
                            Ok(()) => {}
                            Err(err) => {
                                drop(tx.send_err(err));
                                break;
                            }
                        },
                    },
                },
                fib::Complete(value) => {
                    match convert(value) {
                        Ok(None) => {}
                        Ok(Some(value)) => match tx.send(value) {
                            Ok(()) => {}
                            Err(SendError { value, kind }) => match kind {
                                SendErrorKind::Canceled => {}
                                SendErrorKind::Overflow => match overflow(value) {
                                    Ok(()) => {}
                                    Err(err) => {
                                        drop(tx.send_err(err));
                                    }
                                },
                            },
                        },
                        Err(err) => {
                            drop(tx.send_err(err));
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

#[inline]
fn add_rx_overwrite<H, F, T, E, C>(
    thr: H,
    capacity: usize,
    mut fib: F,
    convert: C,
) -> Receiver<T, E>
where
    H: ThrToken,
    F: Fiber<Input = (), Yield = Option<T>>,
    C: FnOnce(F::Return) -> Result<Option<T>, E>,
    F: Send + 'static,
    T: Send + 'static,
    E: Send + 'static,
    C: Send + 'static,
{
    let (mut tx, rx) = channel(capacity);
    thr.add(move || {
        loop {
            if tx.is_canceled() {
                break;
            }
            match unsafe { Pin::new_unchecked(&mut fib) }.resume(()) {
                fib::Yielded(None) => {}
                fib::Yielded(Some(value)) => match tx.send_overwrite(value) {
                    Ok(()) => (),
                    Err(_) => break,
                },
                fib::Complete(value) => {
                    match convert(value) {
                        Ok(None) => {}
                        Ok(Some(value)) => {
                            drop(tx.send_overwrite(value));
                        }
                        Err(err) => {
                            drop(tx.send_err(err));
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

impl<T: ThrToken> ThrFiberStreamRing for T {}
