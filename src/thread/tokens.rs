use thread::prelude::*;

/// A set of thread tokens.
pub trait ThreadTokens {
  /// Thread array.
  type Thread: Thread;

  /// Thread register tokens.
  type Token;

  /// Creates a new set of thread tokens.
  fn new(token: Self::Token) -> Self;
}
