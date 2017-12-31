/// A set of register tokens.
pub trait RegTokens {
  /// Creates a new set of register tokens.
  ///
  /// # Safety
  ///
  /// * Must be called no more than once.
  /// * Must be called at the very beginning of the program flow.
  unsafe fn new() -> Self;
}
