//! The Threads prelude.
//!
//! The purpose of this module is to alleviate imports of many common thread
//! token traits by adding a glob import to the top of thread token heavy
//! modules:
//!
//! ```
//! # #![allow(unused_imports)]
//! use drone_core::reg::prelude::*;
//! ```

pub use crate::thr::ThrToken;

pub use crate::fib::{
    ThrFiberFn as _, ThrFiberFuture as _, ThrFiberGen as _, ThrStreamRing as _, ThrStreamUnit as _,
};
