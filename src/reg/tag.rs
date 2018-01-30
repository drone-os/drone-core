/// Register token tag.
pub trait RegTag: Sized + Send + Sync + Default + 'static {}

/// Owned register token tag.
pub trait RegOwned: RegTag {}

/// Atomic register token tag.
pub trait RegAtomic: RegTag {}

/// Unsynchronized register token tag.
#[derive(Default)]
pub struct Urt;

impl RegTag for Urt {}
impl RegOwned for Urt {}

/// Synchronized register token tag.
#[derive(Default)]
pub struct Srt;

impl RegTag for Srt {}
impl RegOwned for Srt {}
impl RegAtomic for Srt {}

/// Forkable register token tag.
#[derive(Default)]
pub struct Frt;

impl RegTag for Frt {}
impl RegOwned for Frt {}
impl RegAtomic for Frt {}

/// Copyable register token tag.
#[derive(Clone, Copy, Default)]
pub struct Crt;

impl RegTag for Crt {}
impl RegAtomic for Crt {}
