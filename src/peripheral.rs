//! Peripheral devices.

/// Peripheral device.
pub trait PeripheralDevice<T>
where
  Self: Sized + Send + 'static,
  T: PeripheralTokens,
{
  /// Creates a new peripheral from tokens.
  fn from_tokens(tokens: T::InputTokens) -> Self;

  /// Releases the peripheral tokens.
  fn into_tokens(self) -> T;
}

/// Peripheral tokens.
pub trait PeripheralTokens
where
  Self: Sized + Send + 'static,
  Self: From<<Self as PeripheralTokens>::InputTokens>,
{
  /// Input peripheral tokens.
  type InputTokens = Self;
}
