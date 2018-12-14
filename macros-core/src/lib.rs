//! Drone procedural macros shared lib.
//!
//! See `drone-core` documentation for details.

#![warn(missing_docs)]
#![allow(clippy::precedence)]

#[macro_use]
extern crate lazy_static;
extern crate proc_macro;
extern crate proc_macro2;
extern crate regex;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

mod cfg_features;
mod macros;
mod unkeywordize;

pub use self::cfg_features::{CfgFeatures, CfgFeaturesExt};
pub use self::unkeywordize::unkeywordize;
