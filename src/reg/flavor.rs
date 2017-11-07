/// Marker trait for all registers.
pub trait RegFlavor {}

/// Marker trait for owned registers.
pub trait RegOwned: RegFlavor {}

/// Marker trait for shared registers.
pub trait RegShared: RegFlavor {}

/// Zero-sized marker type for **thread-unsafe** register bindings. "Ur" stands
/// for "Unique Register". Does not implement `Send`, `Sync`, 'Clone', 'Copy'.
pub struct Ur;

/// Zero-sized marker type for **thread-safe** register bindings. "Sr" stands
/// for "Shared register". Does implement `Send` and `Sync`, but not 'Clone' and
/// 'Copy'.
pub struct Sr;

/// Zero-sized marker type for **thread-safe** register bindings. "Cr" stands
/// for "Copyable register". Does implement `Send`, `Sync`, 'Clone', 'Copy'.
pub struct Cr;

impl !Sync for Ur {}
impl RegFlavor for Ur {}
impl RegOwned for Ur {}

impl RegFlavor for Sr {}
impl RegOwned for Sr {}
impl RegShared for Sr {}

impl RegFlavor for Cr {}
impl RegShared for Cr {}
