//! The Drone Prelude.
//!
//! This module re-exports:
//! * Contents of [`core::prelude`].
//! * Contents of [`alloc::prelude`].
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

pub use alloc::prelude::v1::*;
pub use core::prelude::v1::*;
