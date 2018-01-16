//! Beginning of program execution.

use core::marker::PhantomData;

/// A token type with only one instance per whole program.
pub struct OriginToken(PhantomData<OriginToken>);

#[cfg_attr(feature = "clippy", allow(new_without_default_derive))]
impl OriginToken {
  /// Creates an instance of the `OriginToken`.
  ///
  /// # Safety
  ///
  /// Must be created at most once per whole program.
  pub unsafe fn new() -> Self {
    OriginToken(PhantomData)
  }
}
