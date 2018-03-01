/// Thread token tag.
pub trait ThrTag: Sized + Clone + Copy + Send + Sync + Default + 'static {}

/// Triggerable thread token tag.
pub trait ThrTrigger: ThrTag {}

/// Controllable thread token tag.
#[derive(Clone, Copy, Default)]
pub struct Ctt;

impl ThrTag for Ctt {}
impl ThrTrigger for Ctt {}

/// Triggerable thread token tag.
#[derive(Clone, Copy, Default)]
pub struct Ttt;

impl ThrTag for Ttt {}
impl ThrTrigger for Ttt {}

/// Limited thread token tag.
#[derive(Clone, Copy, Default)]
pub struct Ltt;

impl ThrTag for Ltt {}
