//! Asynchronous values.
//!
//! This module provides the runtime for `async`/`await` feature. The runtime
//! relies on the Thread-Local Storage feature of [Drone threads](thr) and
//! should be initialized with [`future::init`]. There are two ways to use
//! `async`/`await` in Drone applications:
//!
//! 1. The preferred way is to use `libcore-drone` crate as a dependency. Place
//!    the following to the Cargo.toml:
//!
//!    ```toml
//!    [dependencies]
//!    core = { package = "libcore-drone", version = "0.12.0" }
//!    ```
//!
//!    This way you can use native Rust `async`/`await` syntax.
//!
//! 2. Without `libcore-drone`, attempting to use `.await` will result in the
//!    following errors:
//!
//!    ```text
//!    error[E0433]: failed to resolve: could not find `poll_with_tls_context` in `future`
//!    error[E0433]: failed to resolve: could not find `from_generator` in `future`
//!    ```
//!
//!    You can use [`future::fallback`] module instead. Refer the module
//!    documentation for examples.

pub mod fallback;

mod gen_future;

pub use self::gen_future::from_generator;

use crate::thr::{local, TaskCell, Thread, ThreadLocal};
use core::{
    future::Future,
    mem::transmute,
    pin::Pin,
    sync::atomic::{AtomicUsize, Ordering},
    task::Poll,
};

static LOCAL_TASK_FN: AtomicUsize = AtomicUsize::new(0);

/// Uses the thread-local storage of `T` for the `futures` task system.
///
/// This function should be called before polling any future.
pub fn init<T: Thread>() {
    LOCAL_TASK_FN.store(local_task_fn::<T> as usize, Ordering::Relaxed);
}

/// Polls a future in the current task context.
pub fn poll_with_context<F>(f: Pin<&mut F>) -> Poll<F::Output>
where
    F: Future,
{
    local_task().get_context(|cx| F::poll(f, cx))
}

fn local_task() -> &'static TaskCell {
    let ptr = LOCAL_TASK_FN.load(Ordering::Relaxed);
    if ptr == 0 {
        panic!("drone_core::future::init not called");
    } else {
        unsafe { transmute::<usize, unsafe fn() -> &'static TaskCell>(ptr)() }
    }
}

unsafe fn local_task_fn<T: Thread>() -> &'static TaskCell {
    local::<T>().task()
}
