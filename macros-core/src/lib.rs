//! Core procedural macros crate for [Drone].
//!
//! This crate provides shared functionality for all Drone procedural macro
//! crates.
//!
//! [Drone]: https://github.com/drone-os/drone

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
