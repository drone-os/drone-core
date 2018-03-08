/// Device resource.
pub trait Resource: Sized + Send + 'static {
  /// Source of the resource.
  type Source;

  /// Creates a new resource from the source.
  fn from_source(source: Self::Source) -> Self;
}
