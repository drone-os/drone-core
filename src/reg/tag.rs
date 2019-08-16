//! Register token tags and their traits.
//!
//! The following table shows relations between all types and traits within this
//! module:
//!
//! | Type \ Trait             | [`RegTag`] | [`RegAtomic`] | [`RegOwned`] |
//! |--------------------------|------------|---------------|--------------|
//! | [`Urt`] (Unsynchronized) | **+**      | -             | **+**        |
//! | [`Srt`] (Synchronized)   | **+**      | **+**         | **+**        |
//! | [`Crt`] (Copyable)       | **+**      | **+**         | -            |

/// A register token tag.
///
/// All concrete tags implement this trait.
#[marker]
pub trait RegTag: Sized + Send + Sync + Default + 'static {}

/// An owned register token tag.
///
/// A token tagged with a tag, which implements this trait, follows the
/// move-semantics.
#[marker]
pub trait RegOwned: RegTag {}

/// An atomic register token tag.
///
/// A token tagged with a tag, which implements this trait, uses only atomic
/// operations and can be used concurrently.
#[marker]
pub trait RegAtomic: RegTag {}

/// The unsynchronized register token tag.
///
/// A token tagged with `Urt` cannot be used concurrently and has
/// move-semantics.
#[derive(Default)]
pub struct Urt;

impl RegTag for Urt {}
impl RegOwned for Urt {}

/// The synchronized register token tag.
///
/// A token tagged with `Srt` can be used concurrently and has move-semantics.
#[derive(Default)]
pub struct Srt;

impl RegTag for Srt {}
impl RegOwned for Srt {}
impl RegAtomic for Srt {}

/// The copyable register token tag.
///
/// A token tagged with `Crt` can be used concurrently and implements [`Copy`].
#[derive(Clone, Copy, Default)]
pub struct Crt;

impl RegTag for Crt {}
impl RegAtomic for Crt {}
