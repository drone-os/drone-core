use origin::OriginToken;

/// A set of register tokens.
pub trait RegTokens {
  /// Creates a new set of register tokens.
  fn new(origin: OriginToken) -> Self;
}
