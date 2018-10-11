//! Drone procedural macros shared lib.
//!
//! See `drone-core` documentation for details.

#![feature(proc_macro_diagnostic)]
#![feature(tool_lints)]
#![warn(missing_docs)]
#![allow(clippy::precedence)]
#![doc(html_root_url = "https://docs.rs/drone-macros-core/0.9.0")]

#[macro_use]
extern crate lazy_static;
extern crate proc_macro;
extern crate proc_macro2;
extern crate regex;
#[macro_use]
extern crate syn;

mod extern_fn;
mod extern_static;
mod extern_struct;
mod new_mod;
mod new_static;
mod new_struct;
mod unkeywordize;

pub use self::extern_fn::ExternFn;
pub use self::extern_static::ExternStatic;
pub use self::extern_struct::ExternStruct;
pub use self::new_mod::NewMod;
pub use self::new_static::NewStatic;
pub use self::new_struct::NewStruct;
pub use self::unkeywordize::unkeywordize;
