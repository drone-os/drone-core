//! Traits, helpers, and type definitions for core I/O functionality.
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
//! use drone_core::io;
//! use futures::executor;
//! use futures::future::lazy;
//!
//! struct Buf(Vec<usize>);
//!
//! impl Buf {
//!   fn push(
//!     mut self,
//!     value: usize,
//!   ) -> impl io::Future<
//!     Sess = Self,
//!     Resp = impl for<'r> io::Responder<'r, Self, Output = usize>,
//!     Error = !,
//!   > {
//!     lazy(move || {
//!       self.0.push(value);
//!       Ok((self, |buf: &Buf| buf.0.len()))
//!     })
//!   }
//!
//!   fn push_boxed(
//!     mut self,
//!     value: usize,
//!   ) -> Box<io::Future<
//!     Sess = Self,
//!     Resp = io::NoResp,
//!     Error = !,
//!   >> {
//!     Box::new(lazy(move || {
//!       self.0.push(value);
//!       Ok((self, io::NoResp))
//!     }))
//!   }
//! }
//!
//! fn main() {
//!   let mut executor = executor::spawn(AsyncFuture::new(|| {
//!     let mut buf = Buf(Vec::new());
//!     assert_eq!(ioawait!(buf.push(1))?, 1);
//!     assert_eq!(ioawait!(buf.push(3))?, 2);
//!     assert_eq!(ioawait!(buf.push_boxed(5))?, ());
//!     assert_eq!(ioawait!(buf.push_boxed(7))?, ());
//!     Ok::<_, !>(buf)
//!   }));
//!   loop {
//!     match executor.poll_future_notify(&NOTIFY_NOP, 0) {
//!       Ok(Async::NotReady) => continue,
//!       Ok(Async::Ready(buf)) => {
//!         assert_eq!(buf.0, vec![1, 3, 5, 7]);
//!         break;
//!       }
//!     }
//!   }
//! }
//! ```

mod future;
mod macros;
mod responder;

pub use self::future::{Future, Poll};
pub use self::responder::{NoResp, Responder};
