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
//! use drone_core::prelude::*;
//! ```
//!
//! [`std::prelude`]: https://doc.rust-lang.org/std/prelude/

pub use core::prelude::v1::*;

pub use alloc::{
  borrow::ToOwned,
  boxed::Box,
  slice::SliceConcatExt,
  string::{String, ToString},
  vec::Vec,
};

pub use async::async;
