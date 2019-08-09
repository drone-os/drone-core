//! Asynchronous values.
//!
//! This module provides an API for `async`/`await` feature. There are two ways
//! to use `async`/`await` in Drone applications:
//!
//! 1. The preferred way is to use `drone-async-await` crate as a dependency.
//! Place the following to the Cargo.toml:
//!
//! ```toml
//! [dependencies]
//! core = { package = "drone-async-await", version = "0.9" }
//! ```
//!
//! This way you can use native Rust `async`/`await` syntax.
//!
//! 2. Without `drone-async-await`, attempting to use `.await` will result in
//! the following errors:
//!
//! ```plain
//! error[E0433]: failed to resolve: could not find `poll_with_tls_context` in `future`
//! error[E0433]: failed to resolve: could not find `from_generator` in `future`
//! ```
//!
//! You can use [`future::fallback`] module instead. Refer the module
//! documentation for examples.

pub mod fallback;

mod gen_future;

pub use self::gen_future::from_generator;

use crate::thr::current_task;
use core::{future::Future, pin::Pin, task::Poll};

/// Polls a future in the current task context.
pub fn poll_with_context<F>(f: Pin<&mut F>) -> Poll<F::Output>
where
    F: Future,
{
    current_task().get_context(|cx| F::poll(f, cx))
}
