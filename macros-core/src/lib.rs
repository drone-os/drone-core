//! Drone procedural macros shared lib.
//!
//! See `drone-core` documentation for details.

#![feature(uniform_paths)]
#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

mod cfg_features;
mod macros;
mod unkeywordize;

pub use self::{
  cfg_features::{CfgFeatures, CfgFeaturesExt},
  unkeywordize::unkeywordize,
};
