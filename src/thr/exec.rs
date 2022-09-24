use core::fmt::Display;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, Waker};

use crate::fib;
use crate::thr::prelude::*;

/// Thread executor.
pub trait ThrExec: ThrToken {
    /// Wakes up the thread.
    fn wakeup(self);

    /// Returns a handle for waking up a thread.
    fn waker(self) -> Waker;

    /// Adds an executor for the future `fut` to the fiber chain and wakes up
    /// the thread immediately.
    #[inline]
    fn exec<F, O>(self, fut: F)
    where
        F: Future<Output = O> + Send + 'static,
        O: ExecOutput,
    {
        self.exec_factory(|| fut);
    }

    /// Adds an executor for the future returned by `factory` to the fiber chain
    /// and wakes up the thread immediately.
    #[inline]
    fn exec_factory<C, F, O>(self, factory: C)
    where
        C: FnOnce() -> F + Send + 'static,
        F: Future<Output = O> + 'static,
        O: ExecOutput,
    {
        self.add_exec_factory(factory);
        self.wakeup();
    }

    /// Adds an executor for the future `fut` to the fiber chain.
    ///
    /// The future `fut` will start polling on the next thread wake-up.
    #[inline]
    fn add_exec<F, O>(self, fut: F)
    where
        F: Future<Output = O> + Send + 'static,
        O: ExecOutput,
    {
        self.add_exec_factory(|| fut);
    }

    /// Adds an executor for the future returned by `factory` to the fiber
    /// chain.
    ///
    /// The future `fut` will start polling on the next thread wake-up.
    #[inline]
    fn add_exec_factory<C, F, O>(self, factory: C)
    where
        C: FnOnce() -> F + Send + 'static,
        F: Future<Output = O> + 'static,
        O: ExecOutput,
    {
        fn poll<T: ThrExec, F: Future>(thr: T, fut: Pin<&mut F>) -> Poll<F::Output> {
            let waker = thr.waker();
            let mut cx = Context::from_waker(&waker);
            fut.poll(&mut cx)
        }
        self.add_fn_factory(move || {
            let mut fut = factory();
            move || match poll(self, unsafe { Pin::new_unchecked(&mut fut) }) {
                Poll::Pending => fib::Yielded(()),
                Poll::Ready(output) => {
                    output.terminate();
                    fib::Complete(())
                }
            }
        });
    }
}

/// A trait for implementing arbitrary output types for futures passed to
/// [`ThrExec::exec`] and [`ThrExec::add_exec`].
pub trait ExecOutput: Sized + Send {
    /// The return type of [`ExecOutput::terminate`]. Should be either `()` or
    /// `!`.
    type Terminate;

    /// A result handler for an executor. The returned value will not be used,
    /// so the only useful types are `()` and `!`. The handler may choose to
    /// panic on an erroneous value.
    fn terminate(self) -> Self::Terminate;
}

impl ExecOutput for () {
    type Terminate = ();

    #[inline]
    fn terminate(self) {}
}

#[allow(clippy::mismatching_type_param_order)]
impl<E: Send + Display> ExecOutput for Result<(), E> {
    type Terminate = ();

    #[inline]
    fn terminate(self) {
        match self {
            Ok(()) => {}
            Err(err) => terminate_err(err),
        }
    }
}

impl ExecOutput for ! {
    type Terminate = !;

    #[inline]
    fn terminate(self) -> ! {
        match self {}
    }
}

#[allow(clippy::mismatching_type_param_order)]
impl<E: Send + Display> ExecOutput for Result<!, E> {
    type Terminate = !;

    #[inline]
    fn terminate(self) -> ! {
        let Err(err) = self;
        terminate_err(err);
    }
}

fn terminate_err<E: Display>(err: E) -> ! {
    panic!("root future error: {}", err);
}
