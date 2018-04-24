use futures::prelude::*;

/// The `Read` trait allows for reading bytes from a source.
pub trait Read<'sess> {
  /// The error type for I/O operations.
  type Error;

  /// Pull some bytes from this source into the specified buffer, returning how
  /// many bytes were read.
  fn read(
    &'sess mut self,
    buf: &'sess mut [u8],
  ) -> Box<Future<Item = usize, Error = Self::Error> + 'sess>;
}
