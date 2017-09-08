//! Marker types and traits for register types.

/// Marker trait for various register flavors.
pub trait Flavor {}

/// Zero-sized marker type for **thread-unsafe** registers. Does not implement
/// `Send` and `Sync`.
pub struct Local;

/// Zero-sized marker type for **thread-safe** registers. Does implement `Send`
/// and `Sync`.
pub struct Atomic;

impl !Sync for Local {}

impl Flavor for Local {}

impl Flavor for Atomic {}
