//! Async/await syntax.
//!
//! ```
//! # #![feature(conservative_impl_trait)]
//! # #![feature(decl_macro)]
//! # #![feature(generators)]
//! # #![feature(prelude_import)]
//! # #![feature(proc_macro)]
//! # extern crate drone;
//! # extern crate futures;
//! # #[prelude_import]
//! # use drone::prelude::*;
//! # use futures::executor::Notify;
//! # struct NopNotify;
//! # const NOP_NOTIFY: NopNotify = NopNotify;
//! # impl Notify for NopNotify { fn notify(&self, _id: usize) {} }
//! use drone::async::{async_future, await};
//! use drone::sync::spsc::oneshot;
//! use futures::executor;
//!
//! #[async_future]
//! fn plus_one(
//!   rx: oneshot::Receiver<usize, ()>,
//! ) -> impl Future<Item = usize, Error = oneshot::RecvError<()>> {
//!   let number = await!(rx)?;
//!   Ok(number + 1)
//! }
//!
//! fn main() {
//!   let (tx, rx) = oneshot::channel::<usize, ()>();
//!   let mut executor = executor::spawn(plus_one(rx));
//!   assert_eq!(tx.send(Ok(1)), Ok(()));
//!   assert_eq!(
//!     executor.poll_future_notify(&&NOP_NOTIFY, 0),
//!     Ok(Async::Ready(2))
//!   );
//! }
//! ```

#[doc(hidden)] // FIXME https://github.com/rust-lang/rust/issues/45266
mod async_future;
#[doc(hidden)] // FIXME https://github.com/rust-lang/rust/issues/45266
mod await;

pub use drone_macros::async_future;

pub use self::async_future::AsyncFuture;
pub use self::await::await;
