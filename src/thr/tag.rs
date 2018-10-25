/// Thread token tag.
#[marker]
pub trait ThrTag:
  Sized + Clone + Copy + Send + Sync + Default + 'static
{
}

/// Triggerable thread token tag.
#[marker]
pub trait ThrTrigger: ThrTag {}

/// Unrestricted thread token tag.
#[derive(Clone, Copy, Default)]
pub struct Utt;

impl ThrTag for Utt {}
impl ThrTrigger for Utt {}

/// Triggerable thread token tag.
#[derive(Clone, Copy, Default)]
pub struct Ttt;

impl ThrTag for Ttt {}
impl ThrTrigger for Ttt {}

/// Attachable thread token tag.
#[derive(Clone, Copy, Default)]
pub struct Att;

impl ThrTag for Att {}
