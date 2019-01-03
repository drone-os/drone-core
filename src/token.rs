//! Token - is a ZST, instance of which allows one to manipulate the associated
//! static resource.

pub use drone_core_macros::unsafe_static_tokens;

/// A set of tokens, which represents ownership of static resources.
///
/// # Safety
///
/// * Must be zero-sized.
/// * Contained tokens must not overlap.
pub unsafe trait Tokens: Sized + Send + 'static {
  /// Creates an instance of the token set.
  ///
  /// # Safety
  ///
  /// Must be called no more than once in the program lifetime.
  unsafe fn take() -> Self;
}

/// Token for a static variable.
///
/// # Safety
///
/// * The static must be used only in the trait implementation.
/// * Construction must be possible only via this trait's `take` method.
/// * Must not be `Sync`.
pub unsafe trait StaticToken: Sized + Send + 'static {
  /// The resulting type after dereferencing.
  type Target: ?Sized;

  /// Creates an instance of the static token.
  ///
  /// # Safety
  ///
  /// Caller must take care for synchronizing instances.
  unsafe fn take() -> Self;

  /// Borrows a mutable reference.
  fn get(&mut self) -> &mut Self::Target;

  /// Converts the token into a mutable reference with `'static` lifetime.
  fn into_static(self) -> &'static mut Self::Target;
}
