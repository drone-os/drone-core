//! Fallback syntax for `async`/`await` when `drone-async-await` crate is not
//! available.
//!
//! The following snippet is written with native `async`/`await` syntax:
//!
//! ```
//! # #![feature(async_await)]
//! # #![feature(never_type)]
//! use drone_core::sync::spsc::oneshot;
//!
//! async fn plus_one(rx: oneshot::Receiver<usize>) -> Result<usize, oneshot::Canceled> {
//!     let number = rx.await?;
//!     Ok(number + 1)
//! }
//! ```
//!
//! Using this module that snippet can be rewritten as follow:
//!
//! ```
//! # #![feature(generators)]
//! # #![feature(never_type)]
//! use drone_core::{future::fallback::*, sync::spsc::oneshot};
//! use futures::prelude::*;
//!
//! fn plus_one(
//!     rx: oneshot::Receiver<usize>,
//! ) -> impl Future<Output = Result<usize, oneshot::Canceled>> {
//!     asyn(|| {
//!         let number = awt!(rx)?;
//!         Ok(number + 1)
//!     })
//! }
//! ```

/// `asyn(|| { expr })` is an alternative for `async { expr }`.
pub use super::from_generator as asyn;

/// A macro to await a future on an async call.
///
/// `awt!(expr)` is an alternative for `expr.await`.
pub use crate::awt;

#[doc(hidden)]
#[macro_export]
macro_rules! awt {
    ($expr:expr) => {{
        let mut pinned = $expr;
        loop {
            match $crate::future::poll_with_context(unsafe {
                ::core::pin::Pin::new_unchecked(&mut pinned)
            }) {
                ::core::task::Poll::Ready(x) => break x,
                ::core::task::Poll::Pending => yield,
            }
        }
    }};
}
