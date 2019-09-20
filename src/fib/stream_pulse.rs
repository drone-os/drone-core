use crate::{
    fib::{self, Fiber},
    sync::spsc::pulse::{channel, Receiver, SendError},
    thr::prelude::*,
};
use core::{
    convert::identity,
    num::NonZeroUsize,
    pin::Pin,
    task::{Context, Poll},
};
use futures::Stream;

/// A stream of pulses from the fiber in another thread.
///
/// Dropping or closing this future will remove the fiber on a next thread
/// invocation without resuming it.
#[must_use = "streams do nothing unless you `.await` or poll them"]
pub struct FiberStreamPulse {
    rx: Receiver<!>,
}

/// A fallible stream of pulses from the fiber in another thread.
///
/// Dropping or closing this future will remove the fiber on a next thread
/// invocation without resuming it.
#[must_use = "streams do nothing unless you `.await` or poll them"]
pub struct TryFiberStreamPulse<E> {
    rx: Receiver<E>,
}

impl FiberStreamPulse {
    /// Gracefully close this future.
    ///
    /// The fiber will be removed on a next thread invocation without resuming.
    #[inline]
    pub fn close(&mut self) {
        self.rx.close()
    }
}

impl<E> TryFiberStreamPulse<E> {
    /// Gracefully close this future.
    ///
    /// The fiber will be removed on a next thread invocation without resuming.
    #[inline]
    pub fn close(&mut self) {
        self.rx.close()
    }
}

impl Stream for FiberStreamPulse {
    type Item = NonZeroUsize;

    #[inline]
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let rx = unsafe { self.map_unchecked_mut(|x| &mut x.rx) };
        rx.poll_next(cx).map(|value| value.map(|Ok(value)| value))
    }
}

impl<E> Stream for TryFiberStreamPulse<E> {
    type Item = Result<NonZeroUsize, E>;

    #[inline]
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let rx = unsafe { self.map_unchecked_mut(|x| &mut x.rx) };
        rx.poll_next(cx)
    }
}

/// Extends [`ThrToken`](crate::thr::ThrToken) types with `add_stream_pulse`
/// methods.
pub trait ThrFiberStreamPulse: ThrToken {
    /// Adds the fiber `fib` to the fiber chain and returns a stream of pulses
    /// yielded from the fiber.
    #[inline]
    fn add_stream_pulse_skip<F>(self, fib: F) -> FiberStreamPulse
    where
        F: Fiber<Input = (), Yield = Option<usize>, Return = Option<usize>>,
        F: Send + 'static,
    {
        FiberStreamPulse {
            rx: add_rx(self, || Ok(()), fib, Ok),
        }
    }

    /// Adds the fiber `fib` to the fiber chain and returns a fallible stream of
    /// pulses yielded from the fiber.
    #[inline]
    fn add_stream_pulse<O, F, E>(self, overflow: O, fib: F) -> TryFiberStreamPulse<E>
    where
        O: Fn() -> Result<(), E>,
        F: Fiber<Input = (), Yield = Option<usize>, Return = Result<Option<usize>, E>>,
        O: Send + 'static,
        F: Send + 'static,
        E: Send + 'static,
    {
        TryFiberStreamPulse {
            rx: add_rx(self, overflow, fib, identity),
        }
    }
}

#[inline]
fn add_rx<H, O, F, E, C>(thr: H, overflow: O, mut fib: F, convert: C) -> Receiver<E>
where
    H: ThrToken,
    O: Fn() -> Result<(), E>,
    F: Fiber<Input = (), Yield = Option<usize>>,
    C: FnOnce(F::Return) -> Result<Option<usize>, E>,
    O: Send + 'static,
    F: Send + 'static,
    E: Send + 'static,
    C: Send + 'static,
{
    let (mut tx, rx) = channel();
    thr.add(move || {
        loop {
            if tx.is_canceled() {
                break;
            }
            match unsafe { Pin::new_unchecked(&mut fib) }.resume(()) {
                fib::Yielded(None) => {}
                fib::Yielded(Some(pulses)) => match tx.send(pulses) {
                    Ok(()) => {}
                    Err(SendError::Canceled) => {
                        break;
                    }
                    Err(SendError::Overflow) => match overflow() {
                        Ok(()) => {}
                        Err(err) => {
                            drop(tx.send_err(err));
                            break;
                        }
                    },
                },
                fib::Complete(value) => {
                    match convert(value) {
                        Ok(None) => {}
                        Ok(Some(pulses)) => match tx.send(pulses) {
                            Ok(()) | Err(SendError::Canceled) => {}
                            Err(SendError::Overflow) => match overflow() {
                                Ok(()) => {}
                                Err(err) => {
                                    drop(tx.send_err(err));
                                }
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

impl<T: ThrToken> ThrFiberStreamPulse for T {}
