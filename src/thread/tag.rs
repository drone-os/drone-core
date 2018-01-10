/// Thread token tag.
pub trait ThreadTag: Sized + Clone + Copy + Send + Sync + Default + 'static {}

/// Triggerable thread token tag.
pub trait ThreadTrigger: ThreadTag {}

/// Controllable thread token tag.
#[derive(Clone, Copy, Default)]
pub struct Ctt;

impl ThreadTag for Ctt {}
impl ThreadTrigger for Ctt {}

/// Triggerable thread token tag.
#[derive(Clone, Copy, Default)]
pub struct Ttt;

impl ThreadTag for Ttt {}
impl ThreadTrigger for Ttt {}

/// Limited thread token tag.
#[derive(Clone, Copy, Default)]
pub struct Ltt;

impl ThreadTag for Ltt {}
