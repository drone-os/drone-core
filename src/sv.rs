//! Supervisor.

/// A supervisor interface.
pub trait Supervisor: Sized + 'static {
  /// Returns a pointer to the first service.
  fn first() -> *const Self;
}

/// A supervisor call.
pub trait SvCall<T: SvService>: Supervisor {
  /// Call the system service.
  fn call(service: &mut T);
}

/// A supervisor service.
pub trait SvService: Sized + Send {
  /// A system service handler.
  ///
  /// # Safety
  ///
  /// Must be called only by supervisor.
  unsafe extern "C" fn handler(&mut self);
}
