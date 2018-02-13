use drivers::Resource;

/// Device driver.
pub trait Driver
where
  Self: Sized + Send + Sync + 'static,
{
  /// Device resource.
  type Resource: Resource;

  /// Creates a new driver from the resource `res`.
  fn from_res(res: <Self::Resource as Resource>::Input) -> Self;

  /// Releases the resource.
  fn into_res(self) -> Self::Resource;
}
