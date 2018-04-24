use futures::prelude::*;

/// A trait for objects which are byte-oriented sinks.
pub trait Write<'sess> {
  /// The error type for I/O operations.
  type Error;

  /// Write a buffer into this object, returning how many bytes were written.
  fn write(
    &'sess mut self,
    buf: &'sess [u8],
  ) -> Box<Future<Item = usize, Error = Self::Error> + 'sess>;
}
