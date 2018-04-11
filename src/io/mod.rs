//! Traits, helpers, and type definitions for core I/O functionality.
//!
//! ```
//! # #![feature(const_fn)]
//! # #![feature(exhaustive_patterns)]
//! # #![feature(generators)]
//! # #![feature(prelude_import)]
//! # #![feature(proc_macro)]
//! # #[macro_use] extern crate drone_core;
//! # extern crate futures;
//! # #[prelude_import] use drone_core::prelude::*;
//! # static mut THREADS: [Thr; 1] = [Thr::new(0)];
//! # struct Sv;
//! # struct WakeNop;
//! # unsafe impl task::UnsafeWake for WakeNop {
//! #   unsafe fn clone_raw(&self) -> task::Waker { task::Waker::new(self) }
//! #   unsafe fn drop_raw(&self) {}
//! #   unsafe fn wake(&self) {}
//! # }
//! # impl ::drone_core::sv::Supervisor for Sv {
//! #   fn first() -> *const Self { ::std::ptr::null() }
//! # }
//! # ::drone_core::thr! {
//! #   struct Thr;
//! #   struct ThrLocal;
//! #   extern struct Sv;
//! #   extern static THREADS;
//! # }
//! use drone_core::async::AsyncFuture;
//! use drone_core::io;
//! use futures::prelude::*;
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
//!     lazy(move |_| {
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
//!     Box::new(lazy(move |_| {
//!       self.0.push(value);
//!       Ok((self, io::NoResp))
//!     }))
//!   }
//! }
//!
//! fn main() {
//! # unsafe { drone_core::thr::init::<Thr>() };
//!   let waker = unsafe { task::Waker::new(&WakeNop) };
//!   let mut map = task::LocalMap::new();
//!   let mut cx = task::Context::without_spawn(&mut map, &waker);
//!   let mut fut = AsyncFuture::new(|| {
//!     let mut buf = Buf(Vec::new());
//!     assert_eq!(ioawait!(buf.push(1))?, 1);
//!     assert_eq!(ioawait!(buf.push(3))?, 2);
//!     assert_eq!(ioawait!(buf.push_boxed(5))?, ());
//!     assert_eq!(ioawait!(buf.push_boxed(7))?, ());
//!     Ok::<_, !>(buf)
//!   });
//!   loop {
//!     match fut.poll(&mut cx) {
//!       Ok(Async::Pending) => continue,
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
pub use self::responder::{NoResp, PlainResp, Responder};
