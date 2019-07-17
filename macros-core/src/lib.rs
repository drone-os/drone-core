//! Drone procedural macros shared lib.
//!
//! See `drone-core` documentation for details.

#![deny(bare_trait_objects)]
#![deny(elided_lifetimes_in_paths)]
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
