//! Async/await syntax.
//!
//! ```
//! # #![feature(conservative_impl_trait)]
//! # #![feature(generators)]
//! # #![feature(never_type)]
//! # #![feature(prelude_import)]
//! # #![feature(proc_macro)]
//! # #[macro_use] extern crate drone_core;
//! # extern crate futures;
//! # #[prelude_import] use drone_core::prelude::*;
//! # use futures::executor::Notify;
//! # struct NotifyNop;
//! # const NOTIFY_NOP: &NotifyNop = &NotifyNop;
//! # impl Notify for NotifyNop { fn notify(&self, _id: usize) {} }
//! use drone_core::sync::spsc::oneshot;
//! use futures::executor;
//!
//! fn plus_one(
//!   rx: oneshot::Receiver<usize, !>,
//! ) -> impl Future<Item = usize, Error = oneshot::RecvError<!>> {
//!   AsyncFuture::new(|| {
//!     let number = await!(rx)?;
//!     Ok(number + 1)
//!   })
//! }
//!
//! fn main() {
//!   let (rx, tx) = oneshot::channel::<usize, !>();
//!   let mut executor = executor::spawn(plus_one(rx));
//!   assert_eq!(tx.send(Ok(1)), Ok(()));
//!   assert_eq!(
//!     executor.poll_future_notify(&NOTIFY_NOP, 0).unwrap(),
//!     Async::Ready(2)
//!   );
//! }
//! ```
//!
//! ```
//! # #![feature(conservative_impl_trait)]
//! # #![feature(generators)]
//! # #![feature(never_type)]
//! # #![feature(prelude_import)]
//! # #![feature(proc_macro)]
//! # #[macro_use] extern crate drone_core;
//! # extern crate futures;
//! # #[prelude_import] use drone_core::prelude::*;
//! # use futures::executor::Notify;
//! # struct NotifyNop;
//! # const NOTIFY_NOP: &NotifyNop = &NotifyNop;
//! # impl Notify for NotifyNop { fn notify(&self, _id: usize) {} }
//! use drone_core::sync::spsc::ring;
//! use futures::executor;
//!
//! fn sum(
//!   rx: ring::Receiver<usize, !>,
//! ) -> impl Future<Item = usize, Error = !> {
//!   AsyncFuture::new(|| {
//!     let mut sum = 0;
//!     await_for!(number in rx => {
//!       sum += number;
//!     });
//!     Ok(sum)
//!   })
//! }
//!
//! fn main() {
//!   let (rx, mut tx) = ring::channel::<usize, !>(8);
//!   let mut executor = executor::spawn(sum(rx));
//!   assert_eq!(tx.send_overwrite(3), Ok(()));
//!   assert_eq!(tx.send_overwrite(4), Ok(()));
//!   assert_eq!(tx.send_overwrite(5), Ok(()));
//!   drop(tx);
//!   assert_eq!(
//!     executor.poll_future_notify(&NOTIFY_NOP, 0).unwrap(),
//!     Async::Ready(12)
//!   );
//! }
//! ```

mod async_future;
#[macro_use]
mod await;

pub use self::async_future::AsyncFuture;
