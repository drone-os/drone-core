//! Supervisor.

/// A supervisor interface.
pub trait Supervisor: Sized + 'static {
  /// Returns a pointer to the first service.
  fn first() -> *const Self;
}

/// A supervisor call.
pub trait SvCall<T: SvService>: Supervisor {
  /// Call the system service.
  ///
  /// # Safety
  ///
  /// Directly calling supervisor services is unsafe in general. User code
  /// should use wrappers instead.
  unsafe fn call(service: &mut T);
}

/// A supervisor service.
pub trait SvService: Sized + Send + 'static {
  /// A system service handler.
  ///
  /// # Safety
  ///
  /// Must be called only by supervisor.
  unsafe extern "C" fn handler(&mut self);
}

/// A marker trait for [`SvNone`] or types that implement [`Supervisor`].
#[marker]
pub trait SvOpt {}

/// A type that denotes absence of a supervisor.
pub struct SvNone;

impl<T: Supervisor> SvOpt for T {}

impl SvOpt for SvNone {}
