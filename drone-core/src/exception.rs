//! Exceptions support.

/// Exception routines.
pub trait Exception {
  /// Exception configuration data.
  type Config;

  /// Configures an exception.
  ///
  /// # Safety
  ///
  /// Must be called only by [`exceptions::config`](fn.config.html).
  unsafe fn config(config: Self::Config);

  /// The exception entry.
  fn run(&mut self);
}

/// A vector tables to exception routines.
pub trait ExceptionTable {
  /// Exceptions configuration.
  type Config;

  /// Configures exceptions.
  ///
  /// # Safety
  ///
  /// Must be called no less and no more than once, before exceptions begins
  /// executing.
  unsafe fn config<F>(f: F)
  where
    F: Send + 'static,
    F: FnOnce() -> Self::Config;
}

/// Pointer to an exception routine.
pub type Handler = Option<extern "C" fn()>;

/// Pointer to a reset routine.
pub type ResetHandler = Option<extern "C" fn() -> !>;

/// Reserved vector in a vector table.
#[derive(Clone, Copy)]
#[repr(usize)]
pub enum Reserved {
  Vector = 0,
}
