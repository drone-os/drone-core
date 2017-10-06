//! Reimplementing `core::mem::ManuallyDrop` because the original implementation
//! cannot be used in `const` context.

use core::ops::{Deref, DerefMut};

/// A wrapper to inhibit compiler from automatically calling `T`'s destructor.
#[allow(unions_with_drop_fields)]
pub(crate) union ManuallyDrop<T> {
  value: T,
}

impl<T> ManuallyDrop<T> {
  /// Wrap a value to be manually dropped.
  pub(crate) const fn new(value: T) -> Self {
    Self { value }
  }
}

impl<T> Deref for ManuallyDrop<T> {
  type Target = T;

  #[inline]
  fn deref(&self) -> &Self::Target {
    unsafe { &self.value }
  }
}

impl<T> DerefMut for ManuallyDrop<T> {
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target {
    unsafe { &mut self.value }
  }
}
