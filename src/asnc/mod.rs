//! Async/await syntax.
//!
//! ```
//! # #![feature(const_fn)]
//! # #![feature(futures_api)]
//! # #![feature(generators)]
//! # #![feature(never_type)]
//! # #![feature(prelude_import)]
//! # #[prelude_import] use drone_core::prelude::*;
//! # use core::task::{UnsafeWake, Waker, LocalWaker};
//! # static mut THREADS: [Thr; 1] = [Thr::new(0)];
//! # struct Sv;
//! # struct WakeNop;
//! # unsafe impl UnsafeWake for WakeNop {
//! #   unsafe fn clone_raw(&self) -> Waker { nop_waker().into_waker() }
//! #   unsafe fn drop_raw(&self) {}
//! #   unsafe fn wake(&self) {}
//! # }
//! # impl drone_core::sv::Supervisor for Sv {
//! #   fn first() -> *const Self { std::ptr::null() }
//! # }
//! # fn nop_waker() -> LocalWaker {
//! #   unsafe { LocalWaker::new(core::ptr::NonNull::<WakeNop>::dangling()) }
//! # }
//! # drone_core::thr! {
//! #   struct Thr;
//! #   struct ThrLocal;
//! #   extern struct Sv;
//! #   extern static THREADS;
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
//!   let lw = nop_waker();
//!   let mut fut = Box::pin(plus_one(rx));
//!   assert_eq!(tx.send(Ok(1)), Ok(()));
//!   assert_eq!(Pin::new(&mut fut).poll(&lw), Poll::Ready(Ok(2)));
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
//! # use core::task::{UnsafeWake, Waker, LocalWaker};
//! # static mut THREADS: [Thr; 1] = [Thr::new(0)];
//! # struct Sv;
//! # struct WakeNop;
//! # unsafe impl UnsafeWake for WakeNop {
//! #   unsafe fn clone_raw(&self) -> Waker { nop_waker().into_waker() }
//! #   unsafe fn drop_raw(&self) {}
//! #   unsafe fn wake(&self) {}
//! # }
//! # impl drone_core::sv::Supervisor for Sv {
//! #   fn first() -> *const Self { std::ptr::null() }
//! # }
//! # fn nop_waker() -> LocalWaker {
//! #   unsafe { LocalWaker::new(core::ptr::NonNull::<WakeNop>::dangling()) }
//! # }
//! # drone_core::thr! {
//! #   struct Thr;
//! #   struct ThrLocal;
//! #   extern struct Sv;
//! #   extern static THREADS;
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
//!   let lw = nop_waker();
//!   let mut fut = Box::pin(sum_first_two_items(rx));
//!   assert_eq!(tx.send_overwrite(3), Ok(()));
//!   assert_eq!(tx.send_overwrite(4), Ok(()));
//!   assert_eq!(tx.send_overwrite(5), Ok(()));
//!   drop(tx);
//!   assert_eq!(Pin::new(&mut fut).poll(&lw), Poll::Ready(Ok(7)));
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
//! # use core::task::{UnsafeWake, Waker, LocalWaker};
//! # static mut THREADS: [Thr; 1] = [Thr::new(0)];
//! # struct Sv;
//! # struct WakeNop;
//! # unsafe impl UnsafeWake for WakeNop {
//! #   unsafe fn clone_raw(&self) -> Waker { nop_waker().into_waker() }
//! #   unsafe fn drop_raw(&self) {}
//! #   unsafe fn wake(&self) {}
//! # }
//! # impl drone_core::sv::Supervisor for Sv {
//! #   fn first() -> *const Self { std::ptr::null() }
//! # }
//! # fn nop_waker() -> LocalWaker {
//! #   unsafe { LocalWaker::new(core::ptr::NonNull::<WakeNop>::dangling()) }
//! # }
//! # drone_core::thr! {
//! #   struct Thr;
//! #   struct ThrLocal;
//! #   extern struct Sv;
//! #   extern static THREADS;
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
//!   let lw = nop_waker();
//!   let mut fut = Box::pin(sum(rx));
//!   assert_eq!(tx.send_overwrite(3), Ok(()));
//!   assert_eq!(tx.send_overwrite(4), Ok(()));
//!   assert_eq!(tx.send_overwrite(5), Ok(()));
//!   drop(tx);
//!   assert_eq!(Pin::new(&mut fut).poll(&lw), Poll::Ready(Ok(12)));
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
  current_task().get_waker(|lw| F::poll(f, lw))
}

/// Polls a stream in the current task waker.
pub fn poll_next_with_task_waker<S>(s: Pin<&mut S>) -> Poll<Option<S::Item>>
where
  S: Stream,
{
  current_task().get_waker(|lw| S::poll_next(s, lw))
}
