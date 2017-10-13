//! The building block of threads.
//!
//! See [`Routine`] for more details.
//!
//! [`Routine`]: struct.Routine.html

use core::ops::Generator;
use core::ops::GeneratorState::*;

/// The building block of threads.
pub struct Routine {
  inner: Box<Generator<Yield = (), Return = ()>>,
}

impl<T> From<T> for Routine
where
  T: Generator<Yield = (), Return = ()>,
  T: Send + 'static,
{
  #[inline]
  fn from(generator: T) -> Self {
    let inner = Box::new(generator);
    Self { inner }
  }
}

impl Routine {
  /// Resumes the execution of this routine. Returns `true` if it has finished,
  /// or `false` otherwise.
  #[inline]
  pub fn resume(&mut self) -> bool {
    match self.inner.resume() {
      Yielded(()) => false,
      Complete(()) => true,
    }
  }
}
