//! Async/await syntax.
//!
//! ```
//! # #![feature(const_fn)]
//! # #![feature(futures_api)]
//! # #![feature(generators)]
//! # #![feature(never_type)]
//! # #![feature(prelude_import)]
//! # #[prelude_import] use drone_core::prelude::*;
//! # use core::task::{RawWaker, RawWakerVTable, Waker};
//! # static mut THREADS: [Thr; 1] = [Thr::new(0)];
//! # struct Sv;
//! # impl drone_core::sv::Supervisor for Sv {
//! #   fn first() -> *const Self { core::ptr::null() }
//! # }
//! # drone_core::thr! {
//! #   struct Thr;
//! #   struct ThrLocal;
//! #   extern struct Sv;
//! #   extern static THREADS;
//! # }
//! # fn nop_waker() -> Waker {
//! #   unsafe fn clone(data: *const ()) -> RawWaker {
//! #     RawWaker::new(data, &VTABLE)
//! #   }
//! #   unsafe fn wake(data: *const ()) {}
//! #   static DATA: () = ();
//! #   static VTABLE: RawWakerVTable = RawWakerVTable { clone, wake, drop };
//! #   unsafe { Waker::new_unchecked(RawWaker::new(&DATA, &VTABLE)) }
//! # }
//! use core::{future::Future, pin::Pin, task::Poll};
//! use drone_core::{awt, sync::spsc::oneshot};
//!
//! fn plus_one(
//!   rx: oneshot::Receiver<usize, !>,
//! ) -> impl Future<Output = Result<usize, oneshot::RecvError<!>>> {
//!   asnc(|| {
//!     let number = awt!(rx)?;
//!     Ok(number + 1)
//!   })
//! }
//!
//! fn main() {
//! # unsafe { drone_core::thr::init::<Thr>() };
//!   let (rx, tx) = oneshot::channel::<usize, !>();
//!   let waker = nop_waker();
//!   let mut fut = Box::pin(plus_one(rx));
//!   assert_eq!(tx.send(Ok(1)), Ok(()));
//!   assert_eq!(Pin::new(&mut fut).poll(&waker), Poll::Ready(Ok(2)));
//! }
//! ```
//!
//! ```
//! # #![feature(const_fn)]
//! # #![feature(futures_api)]
//! # #![feature(generators)]
//! # #![feature(never_type)]
//! # #![feature(prelude_import)]
//! # #[prelude_import] use drone_core::prelude::*;
//! # use core::task::{RawWaker, RawWakerVTable, Waker};
//! # static mut THREADS: [Thr; 1] = [Thr::new(0)];
//! # struct Sv;
//! # impl drone_core::sv::Supervisor for Sv {
//! #   fn first() -> *const Self { core::ptr::null() }
//! # }
//! # drone_core::thr! {
//! #   struct Thr;
//! #   struct ThrLocal;
//! #   extern struct Sv;
//! #   extern static THREADS;
//! # }
//! # fn nop_waker() -> Waker {
//! #   unsafe fn clone(data: *const ()) -> RawWaker {
//! #     RawWaker::new(data, &VTABLE)
//! #   }
//! #   unsafe fn wake(data: *const ()) {}
//! #   static DATA: () = ();
//! #   static VTABLE: RawWakerVTable = RawWakerVTable { clone, wake, drop };
//! #   unsafe { Waker::new_unchecked(RawWaker::new(&DATA, &VTABLE)) }
//! # }
//! use drone_core::{awt_next, sync::spsc::ring};
//! use core::{future::Future, pin::Pin, task::Poll};
//!
//! fn sum_first_two_items(
//!   mut rx: ring::Receiver<usize, !>,
//! ) -> impl Future<Output = Result<usize, !>> {
//!   asnc(move || {
//!     if false { yield; }
//!     let a = awt_next!(rx).unwrap_or(Ok(0))?;
//!     let b = awt_next!(rx).unwrap_or(Ok(0))?;
//!     Ok(a + b)
//!   })
//! }
//!
//! fn main() {
//! # unsafe { drone_core::thr::init::<Thr>() };
//!   let (rx, mut tx) = ring::channel::<usize, !>(8);
//!   let waker = nop_waker();
//!   let mut fut = Box::pin(sum_first_two_items(rx));
//!   assert_eq!(tx.send_overwrite(3), Ok(()));
//!   assert_eq!(tx.send_overwrite(4), Ok(()));
//!   assert_eq!(tx.send_overwrite(5), Ok(()));
//!   drop(tx);
//!   assert_eq!(Pin::new(&mut fut).poll(&waker), Poll::Ready(Ok(7)));
//! }
//! ```
//!
//! ```
//! # #![feature(const_fn)]
//! # #![feature(futures_api)]
//! # #![feature(generators)]
//! # #![feature(never_type)]
//! # #![feature(prelude_import)]
//! # #[prelude_import] use drone_core::prelude::*;
//! # use core::task::{RawWaker, RawWakerVTable, Waker};
//! # static mut THREADS: [Thr; 1] = [Thr::new(0)];
//! # struct Sv;
//! # impl drone_core::sv::Supervisor for Sv {
//! #   fn first() -> *const Self { core::ptr::null() }
//! # }
//! # drone_core::thr! {
//! #   struct Thr;
//! #   struct ThrLocal;
//! #   extern struct Sv;
//! #   extern static THREADS;
//! # }
//! # fn nop_waker() -> Waker {
//! #   unsafe fn clone(data: *const ()) -> RawWaker {
//! #     RawWaker::new(data, &VTABLE)
//! #   }
//! #   unsafe fn wake(data: *const ()) {}
//! #   static DATA: () = ();
//! #   static VTABLE: RawWakerVTable = RawWakerVTable { clone, wake, drop };
//! #   unsafe { Waker::new_unchecked(RawWaker::new(&DATA, &VTABLE)) }
//! # }
//! use drone_core::{awt_for, sync::spsc::ring};
//! use core::{future::Future, pin::Pin, task::Poll};
//!
//! fn sum(
//!   rx: ring::Receiver<usize, !>,
//! ) -> impl Future<Output = Result<usize, !>> {
//!   asnc(|| {
//!     let mut sum = 0;
//!     awt_for!(number in rx => {
//!       sum += number?;
//!     });
//!     Ok(sum)
//!   })
//! }
//!
//! fn main() {
//! # unsafe { drone_core::thr::init::<Thr>() };
//!   let (rx, mut tx) = ring::channel::<usize, !>(8);
//!   let waker = nop_waker();
//!   let mut fut = Box::pin(sum(rx));
//!   assert_eq!(tx.send_overwrite(3), Ok(()));
//!   assert_eq!(tx.send_overwrite(4), Ok(()));
//!   assert_eq!(tx.send_overwrite(5), Ok(()));
//!   drop(tx);
//!   assert_eq!(Pin::new(&mut fut).poll(&waker), Poll::Ready(Ok(12)));
//! }
//! ```

mod gen_future;
#[macro_use]
mod macros;

#[doc(hidden)]
pub mod __rt {
  pub use core::{option::Option, pin::Pin, task::Poll};
}

pub use self::gen_future::asnc;

use crate::thr::current_task;
use core::{future::Future, pin::Pin, task::Poll};
use futures::stream::Stream;

/// Polls a future in the current task waker.
pub fn poll_with_task_waker<F>(f: Pin<&mut F>) -> Poll<F::Output>
where
  F: Future,
{
  current_task().get_waker(|waker| F::poll(f, waker))
}

/// Polls a stream in the current task waker.
pub fn poll_next_with_task_waker<S>(s: Pin<&mut S>) -> Poll<Option<S::Item>>
where
  S: Stream,
{
  current_task().get_waker(|waker| S::poll_next(s, waker))
}
