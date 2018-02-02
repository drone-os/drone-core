//! Device drivers.

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

/// Device resource.
pub trait Resource
where
  Self: Sized + Send + Sync + 'static,
  Self: From<<Self as Resource>::Input>,
{
  /// Input resource.
  type Input = Self;
}
