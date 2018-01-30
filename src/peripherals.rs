//! Peripheral devices.

/// Peripheral device.
pub trait PeripheralDevice
where
  Self: Sized + Send + Sync + 'static,
{
  /// Peripheral tokens.
  type Tokens: PeripheralTokens;

  /// Creates a new peripheral from tokens.
  fn from_tokens(
    tokens: <Self::Tokens as PeripheralTokens>::InputTokens,
  ) -> Self;

  /// Releases the peripheral tokens.
  fn into_tokens(self) -> Self::Tokens;
}

/// Peripheral tokens.
pub trait PeripheralTokens
where
  Self: Sized + Send + Sync + 'static,
  Self: From<<Self as PeripheralTokens>::InputTokens>,
{
  /// Input peripheral tokens.
  type InputTokens = Self;
}
