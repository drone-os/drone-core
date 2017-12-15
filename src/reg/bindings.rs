/// A set of register bindings.
pub trait RegBindings {
  /// Creates a new set of register bindings.
  ///
  /// # Safety
  ///
  /// * Must be called no more than once.
  /// * Must be called at the very beginning of the program flow.
  unsafe fn new() -> Self;
}
