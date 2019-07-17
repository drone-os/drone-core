//! Timers.

use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use failure::Fail;
use futures::stream::Stream;

/// Error returned from [`Timer::interval`] on overflow.
#[derive(Debug, Fail)]
#[fail(display = "Timer stream overflow.")]
pub struct TimerOverflow;

/// Generic timer driver.
pub trait Timer: Send {
    /// Timer stop handler.
    type Stop: TimerStop;

    /// Returns a future which resolves when `duration` time elapsed.
    fn sleep(&mut self, duration: usize) -> TimerSleep<'_, Self::Stop>;

    /// Returns a unit stream for timer tick events of `duration` interval.
    /// Returns [`TimerOverflow`] on overflow.
    fn interval(
        &mut self,
        duration: usize,
    ) -> TimerInterval<'_, Self::Stop, Result<(), TimerOverflow>>;

    /// Returns a unit stream for timer tick events of `duration` interval.
    /// Overflows are suppressed.
    fn interval_skip(&mut self, duration: usize) -> TimerInterval<'_, Self::Stop, ()>;
}

/// Timer stop handler.
pub trait TimerStop: Send {
    /// Stops the timer.
    fn stop(&mut self);
}

/// Future created from [`Timer::sleep`].
pub struct TimerSleep<'a, T: TimerStop> {
    stop: &'a mut T,
    future: Pin<Box<dyn Future<Output = ()> + Send + 'a>>,
}

/// Stream created from [`Timer::interval`] or  [`Timer::interval_skip`].
pub struct TimerInterval<'a, T: TimerStop, I> {
    stop: &'a mut T,
    stream: Pin<Box<dyn Stream<Item = I> + Send + 'a>>,
}

impl<'a, T: TimerStop> TimerSleep<'a, T> {
    /// Creates a new [`TimerSleep`].
    pub fn new(stop: &'a mut T, future: Pin<Box<dyn Future<Output = ()> + Send + 'a>>) -> Self {
        Self { stop, future }
    }
}

impl<'a, T: TimerStop> Future for TimerSleep<'a, T> {
    type Output = ();

    #[inline]
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        self.future.as_mut().poll(cx)
    }
}

impl<'a, T: TimerStop> Drop for TimerSleep<'a, T> {
    #[inline]
    fn drop(&mut self) {
        self.stop.stop();
    }
}

impl<'a, T: TimerStop, I> TimerInterval<'a, T, I> {
    /// Creates a new [`TimerInterval`].
    pub fn new(stop: &'a mut T, stream: Pin<Box<dyn Stream<Item = I> + Send + 'a>>) -> Self {
        Self { stop, stream }
    }

    /// Stops the timer and the stream.
    #[inline]
    pub fn stop(mut self: Pin<&mut Self>) {
        self.stop.stop();
    }
}

impl<'a, T: TimerStop, I> Stream for TimerInterval<'a, T, I> {
    type Item = I;

    #[inline]
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<I>> {
        self.stream.as_mut().poll_next(cx)
    }
}

impl<'a, T: TimerStop, I> Drop for TimerInterval<'a, T, I> {
    #[inline]
    fn drop(&mut self) {
        self.stop.stop();
    }
}
