/// Register token tag.
pub trait RegTag: Sized + Default + 'static {}

/// Owned register token tag.
pub trait RegOwned: RegTag {}

/// Shared register token tag.
pub trait RegShared: RegTag {}

/// Unique register token tag.
#[derive(Default)]
pub struct Urt;

impl !Sync for Urt {}
impl RegTag for Urt {}
impl RegOwned for Urt {}

/// Synchronous register token tag.
#[derive(Default)]
pub struct Srt;

impl RegTag for Srt {}
impl RegOwned for Srt {}
impl RegShared for Srt {}

/// Forkable register token tag.
#[derive(Default)]
pub struct Frt;

impl RegTag for Frt {}
impl RegOwned for Frt {}
impl RegShared for Frt {}

/// Copyable register token tag.
#[derive(Clone, Copy, Default)]
pub struct Crt;

impl RegTag for Crt {}
impl RegShared for Crt {}
