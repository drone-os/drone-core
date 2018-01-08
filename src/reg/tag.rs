/// Token tag.
pub trait RegTag: Sized + Default + 'static {}

/// Owned token tag.
pub trait RegOwned: RegTag {}

/// Shared token tag.
pub trait RegShared: RegTag {}

/// Unique token tag.
#[derive(Default)]
pub struct Utt;

impl !Sync for Utt {}
impl RegTag for Utt {}
impl RegOwned for Utt {}

/// Synchronous token tag.
#[derive(Default)]
pub struct Stt;

impl RegTag for Stt {}
impl RegOwned for Stt {}
impl RegShared for Stt {}

/// Forkable token tag.
#[derive(Default)]
pub struct Ftt;

impl RegTag for Ftt {}
impl RegOwned for Ftt {}
impl RegShared for Ftt {}

/// Copyable token tag.
#[derive(Clone, Copy, Default)]
pub struct Ctt;

impl RegTag for Ctt {}
impl RegShared for Ctt {}
