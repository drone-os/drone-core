/// Register tag.
pub trait RegTag: Sized + Default {}

/// Owned register tag.
pub trait RegOwned: RegTag {}

/// Shared register tag.
pub trait RegShared: RegTag {}

/// Unique register tag.
#[derive(Default)]
pub struct Urt;

impl !Sync for Urt {}
impl RegTag for Urt {}
impl RegOwned for Urt {}

/// Synchronous register tag.
#[derive(Default)]
pub struct Srt;

impl RegTag for Srt {}
impl RegOwned for Srt {}
impl RegShared for Srt {}

/// Duplicable register tag.
#[derive(Default)]
pub struct Drt;

impl RegTag for Drt {}
impl RegOwned for Drt {}
impl RegShared for Drt {}

/// Copyable register tag.
#[derive(Clone, Copy, Default)]
pub struct Crt;

impl RegTag for Crt {}
impl RegShared for Crt {}
