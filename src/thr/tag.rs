/// Thread token tag.
#[marker]
pub trait ThrTag:
  Sized + Clone + Copy + Send + Sync + Default + 'static
{
}

/// Attachable thread token tag.
#[marker]
pub trait ThrAttach: ThrTag {}

/// Triggerable thread token tag.
#[marker]
pub trait ThrTrigger: ThrTag {}

/// Attach-only thread token tag.
#[derive(Clone, Copy, Default)]
pub struct Att;

impl ThrTag for Att {}
impl ThrAttach for Att {}

/// Trigger-only thread token tag.
#[derive(Clone, Copy, Default)]
pub struct Ttt;

impl ThrTag for Ttt {}
impl ThrTrigger for Ttt {}

/// Regular thread token tag.
#[derive(Clone, Copy, Default)]
pub struct Rtt;

impl ThrTag for Rtt {}
impl ThrAttach for Rtt {}
impl ThrTrigger for Rtt {}

/// Controllable thread token tag.
#[derive(Clone, Copy, Default)]
pub struct Ctt;

impl ThrTag for Ctt {}
impl ThrAttach for Ctt {}
impl ThrTrigger for Ctt {}
