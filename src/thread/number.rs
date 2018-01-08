/// Thread token.
pub trait ThreadNumber: Sized + Send + Sync + 'static {
  /// A thread position within threads array.
  const THREAD_NUMBER: usize;
}
