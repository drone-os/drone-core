/// Thread token tag.
#[marker]
pub trait ThrTag:
  Sized + Clone + Copy + Send + Sync + Default + 'static
{
}

/// Attachable thread token tag.
#[marker]
pub trait ThrAttach: ThrTag {}

/// Trigger-only thread token tag.
#[derive(Clone, Copy, Default)]
pub struct Ttt;

impl ThrTag for Ttt {}

/// Attach and trigger thread token tag.
#[derive(Clone, Copy, Default)]
pub struct Att;

impl ThrTag for Att {}
impl ThrAttach for Att {}

/// Privileged thread token tag.
#[derive(Clone, Copy, Default)]
pub struct Ptt;

impl ThrTag for Ptt {}
impl ThrAttach for Ptt {}
