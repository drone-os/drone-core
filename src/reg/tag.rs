/// Binding tag.
pub trait RegTag: Sized + Default {}

/// Owned binding tag.
pub trait RegOwned: RegTag {}

/// Shared binding tag.
pub trait RegShared: RegTag {}

/// Unique binding tag.
#[derive(Default)]
pub struct Ubt;

impl !Sync for Ubt {}
impl RegTag for Ubt {}
impl RegOwned for Ubt {}

/// Synchronous binding tag.
#[derive(Default)]
pub struct Sbt;

impl RegTag for Sbt {}
impl RegOwned for Sbt {}
impl RegShared for Sbt {}

/// Forkable binding tag.
#[derive(Default)]
pub struct Fbt;

impl RegTag for Fbt {}
impl RegOwned for Fbt {}
impl RegShared for Fbt {}

/// Copyable binding tag.
#[derive(Clone, Copy, Default)]
pub struct Cbt;

impl RegTag for Cbt {}
impl RegShared for Cbt {}
