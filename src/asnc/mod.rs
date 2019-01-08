//! Async/await syntax.
//!
//! ```
//! # #![feature(const_fn)]
//! # #![feature(generators)]
//! # #![feature(never_type)]
//! # #![feature(prelude_import)]
//! # #[prelude_import] use drone_core::prelude::*;
//! # static mut THREADS: [Thr; 1] = [Thr::new(0)];
//! # struct Sv;
//! # struct WakeNop;
//! # unsafe impl task::UnsafeWake for WakeNop {
//! #   unsafe fn clone_raw(&self) -> task::Waker { task::Waker::new(self) }
//! #   unsafe fn drop_raw(&self) {}
//! #   unsafe fn wake(&self) {}
//! # }
//! # impl drone_core::sv::Supervisor for Sv {
//! #   fn first() -> *const Self { std::ptr::null() }
//! # }
//! # drone_core::thr! {
//! #   struct Thr;
//! #   struct ThrLocal;
//! #   extern struct Sv;
//! #   extern static THREADS;
//! # }
//! use drone_core::{awt, sync::spsc::oneshot};
//! use futures::prelude::*;
//!
//! fn plus_one(
//!   rx: oneshot::Receiver<usize, !>,
//! ) -> impl Future<Item = usize, Error = oneshot::RecvError<!>> {
//!   asnc(|| {
//!     let number = awt!(rx)?;
//!     Ok(number + 1)
//!   })
//! }
//!
//! fn main() {
//! # unsafe { drone_core::thr::init::<Thr>() };
//!   let (rx, tx) = oneshot::channel::<usize, !>();
//!   let waker = unsafe { task::Waker::new(&WakeNop) };
//!   let mut map = task::LocalMap::new();
//!   let mut cx = task::Context::without_spawn(&mut map, &waker);
//!   let mut fut = plus_one(rx);
//!   assert_eq!(tx.send(Ok(1)), Ok(()));
//!   assert_eq!(fut.poll(&mut cx).unwrap(), Async::Ready(2));
//! }
//! ```
//!
//! ```
//! # #![feature(const_fn)]
//! # #![feature(generators)]
//! # #![feature(never_type)]
//! # #![feature(prelude_import)]
//! # #[prelude_import] use drone_core::prelude::*;
//! # static mut THREADS: [Thr; 1] = [Thr::new(0)];
//! # struct Sv;
//! # struct WakeNop;
//! # unsafe impl task::UnsafeWake for WakeNop {
//! #   unsafe fn clone_raw(&self) -> task::Waker { task::Waker::new(self) }
//! #   unsafe fn drop_raw(&self) {}
//! #   unsafe fn wake(&self) {}
//! # }
//! # impl drone_core::sv::Supervisor for Sv {
//! #   fn first() -> *const Self { std::ptr::null() }
//! # }
//! # drone_core::thr! {
//! #   struct Thr;
//! #   struct ThrLocal;
//! #   extern struct Sv;
//! #   extern static THREADS;
//! # }
//! use drone_core::{awt_item, sync::spsc::ring};
//! use futures::prelude::*;
//!
//! fn sum_first_two_items(
//!   mut rx: ring::Receiver<usize, !>,
//! ) -> impl Future<Item = usize, Error = !> {
//!   asnc(move || {
//!     if false { yield; }
//!     let a = awt_item!(rx)?.unwrap_or(0);
//!     let b = awt_item!(rx)?.unwrap_or(0);
//!     Ok(a + b)
//!   })
//! }
//!
//! fn main() {
//! # unsafe { drone_core::thr::init::<Thr>() };
//!   let (rx, mut tx) = ring::channel::<usize, !>(8);
//!   let waker = unsafe { task::Waker::new(&WakeNop) };
//!   let mut map = task::LocalMap::new();
//!   let mut cx = task::Context::without_spawn(&mut map, &waker);
//!   let mut fut = sum_first_two_items(rx);
//!   assert_eq!(tx.send_overwrite(3), Ok(()));
//!   assert_eq!(tx.send_overwrite(4), Ok(()));
//!   assert_eq!(tx.send_overwrite(5), Ok(()));
//!   drop(tx);
//!   assert_eq!(fut.poll(&mut cx).unwrap(), Async::Ready(7));
//! }
//! ```
//!
//! ```
//! # #![feature(const_fn)]
//! # #![feature(generators)]
//! # #![feature(never_type)]
//! # #![feature(prelude_import)]
//! # #[prelude_import] use drone_core::prelude::*;
//! # static mut THREADS: [Thr; 1] = [Thr::new(0)];
//! # struct Sv;
//! # struct WakeNop;
//! # unsafe impl task::UnsafeWake for WakeNop {
//! #   unsafe fn clone_raw(&self) -> task::Waker { task::Waker::new(self) }
//! #   unsafe fn drop_raw(&self) {}
//! #   unsafe fn wake(&self) {}
//! # }
//! # impl drone_core::sv::Supervisor for Sv {
//! #   fn first() -> *const Self { std::ptr::null() }
//! # }
//! # drone_core::thr! {
//! #   struct Thr;
//! #   struct ThrLocal;
//! #   extern struct Sv;
//! #   extern static THREADS;
//! # }
//! use drone_core::{awt_for, sync::spsc::ring};
//! use futures::prelude::*;
//!
//! fn sum(
//!   rx: ring::Receiver<usize, !>,
//! ) -> impl Future<Item = usize, Error = !> {
//!   asnc(|| {
//!     let mut sum = 0;
//!     awt_for!(number in rx => {
//!       sum += number;
//!     });
//!     Ok(sum)
//!   })
//! }
//!
//! fn main() {
//! # unsafe { drone_core::thr::init::<Thr>() };
//!   let (rx, mut tx) = ring::channel::<usize, !>(8);
//!   let waker = unsafe { task::Waker::new(&WakeNop) };
//!   let mut map = task::LocalMap::new();
//!   let mut cx = task::Context::without_spawn(&mut map, &waker);
//!   let mut fut = sum(rx);
//!   assert_eq!(tx.send_overwrite(3), Ok(()));
//!   assert_eq!(tx.send_overwrite(4), Ok(()));
//!   assert_eq!(tx.send_overwrite(5), Ok(()));
//!   drop(tx);
//!   assert_eq!(fut.poll(&mut cx).unwrap(), Async::Ready(12));
//! }
//! ```

mod gen_future;
#[macro_use]
mod macros;

#[doc(hidden)]
pub mod __rt {
  pub use core::{option::Option, result::Result};
  pub use futures::{Async, Future, Stream};
}

pub use self::gen_future::asnc;
