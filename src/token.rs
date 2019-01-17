//! Token - is a ZST, instance of which allows one to manipulate the associated
//! static resource.

pub use drone_core_macros::{unsafe_init_tokens, unsafe_static_tokens};

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
  /// Calling the method more than once in the program lifetime is not safe.
  unsafe fn take() -> Self;
}

/// Token for a one-time action, e.g. an initializer.
///
/// # Safety
///
/// Construction must be possible only via this trait's `take` method.
pub unsafe trait InitToken: Sized + Send + 'static {
  /// Creates an instance of the init token.
  ///
  /// # Safety
  ///
  /// Calling the method more than once in the program lifetime is not safe.
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
  /// Calling the method more than once in the program lifetime is not safe.
  unsafe fn take() -> Self;

  /// Borrows a mutable reference.
  fn get(&mut self) -> &mut Self::Target;

  /// Converts the token into a mutable reference with `'static` lifetime.
  fn into_static(self) -> &'static mut Self::Target;
}

/// Defines a new [`InitToken`].
#[macro_export]
macro_rules! init_token {
  ($(#[$attr:meta])* $vis:vis struct $ident:ident $(;)*) => {
    $(#[$attr])* $vis struct $ident(());

    unsafe impl $crate::token::InitToken for $ident {
      #[inline(always)]
      unsafe fn take() -> Self {
        $ident(())
      }
    }
  };
}
