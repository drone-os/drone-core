//! Traits, helpers, and type definitions for core I/O functionality.

mod read;
mod seek;
mod write;

pub use self::{
    read::Read,
    seek::{Seek, SeekFrom},
    write::Write,
};
