/// Thread token tag.
pub trait ThdTag: Sized + Clone + Copy + Send + Sync + Default + 'static {}

/// Triggerable thread token tag.
pub trait ThdTrigger: ThdTag {}

/// Controllable thread token tag.
#[derive(Clone, Copy, Default)]
pub struct Ctt;

impl ThdTag for Ctt {}
impl ThdTrigger for Ctt {}

/// Triggerable thread token tag.
#[derive(Clone, Copy, Default)]
pub struct Ttt;

impl ThdTag for Ttt {}
impl ThdTrigger for Ttt {}

/// Limited thread token tag.
#[derive(Clone, Copy, Default)]
pub struct Ltt;

impl ThdTag for Ltt {}
