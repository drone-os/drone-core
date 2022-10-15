//! The Drone Prelude.
//!
//! By default Rust automatically injects libcore prelude imports into every
//! module. To inject the Drone prelude instead, place the following code to the
//! `src/lib.rs`:
//!
//! ```
//! #![feature(prelude_import)]
//!
//! #[prelude_import]
//! #[allow(unused_imports)]
//! use drone_core::prelude::*;
//! ```

#[cfg(not(feature = "std"))]
#[doc(no_inline)]
pub use crate::{dbg, eprint, eprintln, print, println};
#[doc(no_inline)]
pub use alloc::{
    borrow::ToOwned,
    boxed::Box,
    string::{String, ToString},
    vec::Vec,
};
#[doc(no_inline)]
pub use core::prelude::rust_2021::*;
#[cfg(feature = "std")]
#[doc(no_inline)]
pub use std::{dbg, eprint, eprintln, print, println};
