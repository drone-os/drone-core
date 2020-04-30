//! Traits, helpers, and type definitions for core I/O functionality.
//!
//! The module contains a number of common things you'll need when doing input
//! and output. The most core part of this module is the [`Read`] and [`Write`]
//! traits, which provide the most general interface for reading and writing
//! input and output.

mod read;
mod seek;
mod write;

pub use self::{
    read::Read,
    seek::{Seek, SeekFrom},
    write::Write,
};
