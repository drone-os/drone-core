/// Marker trait for various register flavors.
pub trait RegFlavor {}

/// Zero-sized marker type for **thread-unsafe** register bindings. "Lr" stands
/// for "Local Register". Does not implement `Send`, `Sync`, 'Clone', 'Copy'.
pub struct Lr;

/// Zero-sized marker type for **thread-safe** register bindings. "Ar" stands
/// for "Atomic register". Does implement `Send`, `Sync`, 'Clone', 'Copy'.
pub struct Ar;

impl !Sync for Lr {}

impl RegFlavor for Lr {}

impl RegFlavor for Ar {}
