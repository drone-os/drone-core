//! Single-producer, single-consumer queues.

use core::{
    mem::MaybeUninit,
    ops::{BitAnd, BitOr, BitOrAssign, BitXorAssign},
    sync::atomic::Ordering,
    task::{Context, Poll, Waker},
};

pub mod oneshot;
pub mod ring;
pub mod unit;

pub(self) trait SpscInner<A, I>
where
    I: Copy + Eq + BitAnd<Output = I> + BitOr<Output = I> + BitOrAssign + BitXorAssign,
{
    const ZERO: I;
    const RX_WAKER_STORED: I;
    const TX_WAKER_STORED: I;
    const COMPLETE: I;

    fn state_load(&self, order: Ordering) -> I;

    fn compare_exchange_weak(
        &self,
        current: I,
        new: I,
        success: Ordering,
        failure: Ordering,
    ) -> Result<I, I>;

    #[allow(clippy::mut_from_ref)]
    unsafe fn rx_waker_mut(&self) -> &mut MaybeUninit<Waker>;

    #[allow(clippy::mut_from_ref)]
    unsafe fn tx_waker_mut(&self) -> &mut MaybeUninit<Waker>;

    #[inline]
    fn transaction<R, E>(
        &self,
        mut old: I,
        success: Ordering,
        failure: Ordering,
        f: impl Fn(&mut I) -> Result<R, E>,
    ) -> Result<R, E> {
        loop {
            let mut new = old;
            let result = f(&mut new);
            if result.is_err() {
                break result;
            }
            match self.compare_exchange_weak(old, new, success, failure) {
                Ok(_) => break result,
                Err(x) => old = x,
            }
        }
    }

    #[inline]
    fn is_canceled(&self, order: Ordering) -> bool {
        let state = self.state_load(order);
        self.take_cancel(state).is_ready()
    }

    fn take_cancel(&self, state: I) -> Poll<()> {
        if state & Self::COMPLETE == Self::ZERO {
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }

    fn poll_half<T>(
        &self,
        cx: &mut Context<'_>,
        is_tx_half: bool,
        read_order: Ordering,
        cas_order: Ordering,
        take: fn(&Self, I) -> Poll<T>,
    ) -> Poll<T> {
        let waker_stored = if is_tx_half {
            Self::TX_WAKER_STORED
        } else {
            Self::RX_WAKER_STORED
        };
        let state = self.state_load(read_order);
        let value = take(self, state);
        if value.is_ready() || state & waker_stored != Self::ZERO {
            return value;
        }
        unsafe {
            let waker = if is_tx_half {
                self.tx_waker_mut()
            } else {
                self.rx_waker_mut()
            };
            waker.write(cx.waker().clone());
        }
        let Ok(state) = self.transaction(state, cas_order, read_order, |state| {
            *state |= waker_stored;
            Ok::<_, !>(*state)
        });
        take(self, state)
    }

    fn poll_half_with_transaction<T, U, V>(
        &self,
        cx: &mut Context<'_>,
        is_tx_half: bool,
        read_order: Ordering,
        cas_order: Ordering,
        take_try: fn(&Self, &mut I) -> Option<Result<U, V>>,
        take_finalize: fn(&Self, Result<U, V>) -> T,
    ) -> Poll<T> {
        let waker_stored = if is_tx_half {
            Self::TX_WAKER_STORED
        } else {
            Self::RX_WAKER_STORED
        };
        let state = self.state_load(read_order);
        self.transaction(state, cas_order, read_order, |state| {
            match take_try(self, state) {
                Some(value) => value.map(Ok).map_err(Ok),
                None => Err(Err(if *state & waker_stored == Self::ZERO {
                    Ok(())
                } else {
                    Err(())
                })),
            }
        })
        .or_else(|value| {
            value.map(Err).or_else(|no_waker| {
                no_waker.and_then(|()| {
                    unsafe {
                        let waker = if is_tx_half {
                            self.tx_waker_mut()
                        } else {
                            self.rx_waker_mut()
                        };
                        waker.write(cx.waker().clone());
                    }
                    let Ok(value) = self.transaction(state, cas_order, read_order, |state| {
                        *state |= waker_stored;
                        Ok::<_, !>(take_try(self, state))
                    });
                    value.ok_or(())
                })
            })
        })
        .map_or_else(
            |()| Poll::Pending,
            |value| Poll::Ready(take_finalize(self, value)),
        )
    }

    fn close_half(&self, is_tx_half: bool) {
        let waker_stored = if is_tx_half {
            Self::RX_WAKER_STORED
        } else {
            Self::TX_WAKER_STORED
        };
        let state = self.state_load(Ordering::Acquire);
        if let Ok((waker, complete)) =
            self.transaction(state, Ordering::Acquire, Ordering::Acquire, |state| {
                let waker = if *state & waker_stored == Self::ZERO {
                    false
                } else {
                    *state ^= waker_stored;
                    true
                };
                let complete = if *state & Self::COMPLETE == Self::ZERO {
                    *state |= Self::COMPLETE;
                    true
                } else {
                    false
                };
                if waker || complete {
                    Ok((waker, complete))
                } else {
                    Err(())
                }
            })
        {
            unsafe {
                if waker {
                    let waker = if is_tx_half {
                        self.rx_waker_mut()
                    } else {
                        self.tx_waker_mut()
                    };
                    let waker = waker.read();
                    if complete {
                        waker.wake();
                    }
                }
            }
        }
    }
}

pub(self) trait SpscInnerErr<A, I>: SpscInner<A, I>
where
    I: Copy + Eq + BitAnd<Output = I> + BitOr<Output = I> + BitOrAssign + BitXorAssign,
{
    type Error;

    #[allow(clippy::mut_from_ref)]
    unsafe fn err_mut(&self) -> &mut Option<Self::Error>;

    fn send_err(&self, err: Self::Error) -> Result<(), Self::Error> {
        if self.is_canceled(Ordering::Relaxed) {
            Err(err)
        } else {
            unsafe { *self.err_mut() = Some(err) };
            // Should we do an additional synchronization here?
            Ok(())
        }
    }

    fn take_err<T>(&self) -> Option<Result<T, Self::Error>> {
        unsafe { self.err_mut().take().map(Err) }
    }
}
