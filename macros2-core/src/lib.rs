//! Drone procedural macros shared lib.
//!
//! See `drone-core` documentation for details.

#![warn(missing_docs)]
#![doc(html_root_url = "https://docs.rs/drone-macros2-core/0.8.0")]
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![cfg_attr(feature = "clippy", allow(precedence))]

#[macro_use]
extern crate syn;

mod extern_static;
mod extern_struct;
mod new_static;
mod new_struct;

pub use self::extern_static::ExternStatic;
pub use self::extern_struct::ExternStruct;
pub use self::new_static::NewStatic;
pub use self::new_struct::NewStruct;
