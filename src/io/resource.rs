/// Device resource.
pub trait Resource
where
  Self: Sized + Send + Sync + 'static,
  Self: From<<Self as Resource>::Input>,
{
  /// Input resource.
  type Input = Self;
}
