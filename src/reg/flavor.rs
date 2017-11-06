/// Marker trait for all register flavors.
pub trait RegFlavor {}

/// Marker trait for shared register flavors.
pub trait RegShared: RegFlavor {}

/// Zero-sized marker type for **thread-unsafe** register bindings. "Lr" stands
/// for "Local Register". Does not implement `Send`, `Sync`, 'Clone', 'Copy'.
pub struct Lr;

/// Zero-sized marker type for **thread-safe** register bindings. "Sr" stands
/// for "Shared register". Does implement `Send` and `Sync`, but not 'Clone' and
/// 'Copy'.
pub struct Sr;

/// Zero-sized marker type for **thread-safe** register bindings. "Cr" stands
/// for "Copyable register". Does implement `Send`, `Sync`, 'Clone', 'Copy'.
pub struct Cr;

impl !Sync for Lr {}
impl RegFlavor for Lr {}

impl RegFlavor for Sr {}
impl RegShared for Sr {}

impl RegFlavor for Cr {}
impl RegShared for Cr {}
