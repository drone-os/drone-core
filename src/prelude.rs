//! The Drone Prelude.
//!
//! It is an analogue of [`std::prelude`], which is not available in
//! `#![no_std]` contexts.
//!
//! To automatically inject the imports into every module, place this code to
//! the crate root:
//!
//! ```
//! #![feature(prelude_import)]
//!
//! #[prelude_import]
//! #[allow(unused_imports)]
//! use drone::prelude::*;
//! ```
//!
//! [`std::prelude`]: https://doc.rust-lang.org/std/prelude/

pub use core::prelude::v1::*;

pub use alloc::borrow::ToOwned;
pub use alloc::boxed::Box;
pub use alloc::slice::SliceConcatExt;
pub use alloc::string::{String, ToString};
pub use alloc::vec::Vec;

pub use core::ops::Generator;
pub use core::ops::GeneratorState::*;

pub use futures::{Async, Future, IntoFuture, Poll, Stream};

pub use async::AsyncFuture;
pub use thread::{Thread, ThreadBinding};
