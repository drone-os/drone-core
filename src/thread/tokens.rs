use thread::prelude::*;

/// A set of thread tokens.
pub trait ThreadTokens {
  /// Thread array.
  type Thread: Thread;

  /// Thread register tokens.
  type Tokens;

  /// Creates a new set of thread tokens.
  fn new(tokens: Self::Tokens) -> Self;
}
