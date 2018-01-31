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
//! use drone_core::io::prelude::*;
//! use futures::executor;
//! use futures::future::lazy;
//!
//! struct Buf(Vec<usize>);
//!
//! struct Push(usize);
//!
//! impl<'sess> Operation<'sess> for Push {
//!   type Sess = Buf;
//!   type Arg = usize;
//!   type Item = usize;
//!   type Error = !;
//!
//!   fn new(value: usize) -> Self { Push(value) }
//!
//!   fn operate(&self, mut buf: Buf) -> Box<io::Future<'sess, Self>> {
//!     let value = self.0;
//!     Box::new(lazy(move || {
//!       buf.0.push(value);
//!       Ok(buf)
//!     }))
//!   }
//!
//!   fn respond(self, buf: &'sess Buf) -> usize {
//!     buf.0.len()
//!   }
//! }
//!
//! fn main() {
//!   let mut executor = executor::spawn(AsyncFuture::new(|| {
//!     let mut buf = Buf(Vec::new());
//!     assert_eq!(io_await!(Push, buf, 1)?, 1);
//!     assert_eq!(io_await!(Push, buf, 3)?, 2);
//!     assert_eq!(io_await!(Push, buf, 5)?, 3);
//!     Ok::<_, !>(buf)
//!   }));
//!   loop {
//!     match executor.poll_future_notify(&NOTIFY_NOP, 0) {
//!       Ok(Async::NotReady) => continue,
//!       Ok(Async::Ready(buf)) => {
//!         assert_eq!(buf.0, vec![1, 3, 5]);
//!         break;
//!       }
//!     }
//!   }
//! }
//! ```

pub mod prelude;

#[macro_use]
mod await;

use core::fmt::{self, Debug};
use core::result;
use futures;

/// I/O operation trait. Designed to be used with `io_await!` macro.
pub trait Operation<'sess> {
  /// I/O session.
  type Sess;

  /// Operation argument. If this is a tuple, `io_await!` could be called with
  /// multiple arguments.
  type Arg;

  /// The type of value that this operation will be resolved with if it is
  /// successful.
  type Item;

  /// The type of error that this future will resolve with if it fails.
  type Error: Debug + PartialEq + Eq;

  /// Instantiates a new operation.
  fn new(arg: Self::Arg) -> Self;

  /// Performs the operation.
  fn operate(&self, sess: Self::Sess) -> Box<Future<'sess, Self>>;

  /// Returns a successful result value of the operation.
  fn respond(self, sess: &'sess Self::Sess) -> Self::Item;
}

/// The error type for I/O operations.
pub struct Error<S, K: Debug + PartialEq + Eq> {
  /// I/O session.
  pub sess: S,
  /// Actual kind of the error.
  pub kind: K,
}

/// A specialized `Future` type for I/O operations.
pub type Future<'sess, T> = futures::Future<
  Item = <T as Operation<'sess>>::Sess,
  Error = Error<<T as Operation<'sess>>::Sess, <T as Operation<'sess>>::Error>,
>;

/// A specialized `Result` type for I/O operations.
pub type Result<'sess, T> =
  result::Result<<T as Operation<'sess>>::Item, <T as Operation<'sess>>::Error>;

impl<S, K: Debug + PartialEq + Eq> Error<S, K> {
  /// Creates a new `io::Error` from `sess` and `kind`.
  #[inline(always)]
  pub fn new(sess: S, kind: K) -> Self {
    Error { sess, kind }
  }
}

impl<S, K: Debug + PartialEq + Eq> Debug for Error<S, K> {
  #[inline(always)]
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    self.kind.fmt(f)
  }
}

impl<S, K: Debug + PartialEq + Eq> PartialEq for Error<S, K> {
  #[inline(always)]
  fn eq(&self, other: &Self) -> bool {
    self.kind.eq(&other.kind)
  }
}

impl<S, K: Debug + PartialEq + Eq> Eq for Error<S, K> {}
