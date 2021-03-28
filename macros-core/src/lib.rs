//! Procedural macros base for [Drone], an Embedded Operating System.
//!
//! This crate provides shared functionality for all Drone procedural macro
//! crates.
//!
//! [Drone]: https://github.com/drone-os/drone

#![warn(missing_docs, unsafe_op_in_unsafe_fn)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions, clippy::must_use_candidate)]

mod cfg_cond;
mod macros;
mod unkeywordize;

pub use self::{
    cfg_cond::{CfgCond, CfgCondExt},
    unkeywordize::unkeywordize,
};
