/// Device resource.
pub trait Resource
where
  Self: Sized + Send + Sync + 'static,
  Self: From<<Self as Resource>::Source>,
{
  /// Source resource.
  type Source = Self;
}
