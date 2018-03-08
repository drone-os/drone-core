use drv::Resource;

/// Device driver.
pub trait Driver: Sized + Send + 'static {
  /// Device resource.
  type Resource: Resource;

  /// Creates a new driver from `source`.
  fn new(source: <Self::Resource as Resource>::Source) -> Self;

  /// Releases the resource.
  fn free(self) -> Self::Resource;
}
