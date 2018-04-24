//! Traits, helpers, and type definitions for core I/O functionality.

mod read;
mod write;

pub use self::read::Read;
pub use self::write::Write;
