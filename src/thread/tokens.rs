use thread::prelude::*;

/// A set of thread tokens.
pub trait ThreadTokens<T: Thread> {
  /// Creates a new set of thread tokens.
  ///
  /// # Safety
  ///
  /// * Must be called no more than once.
  /// * Must be called at the very beginning of the program flow.
  unsafe fn new() -> Self;
}
