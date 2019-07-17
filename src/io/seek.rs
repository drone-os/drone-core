use core::{future::Future, pin::Pin};

/// Enumeration of possible methods to seek within an I/O object.
///
/// It is used by the [`Seek`](Seek) trait.
pub enum SeekFrom {
    /// Set the offset to the provided number of bytes.
    Start(u64),
    /// Set the offset to the size of this object plus the specified number of
    /// bytes.
    ///
    /// It is possible to seek beyond the end of an object, but it's an error to
    /// seek before byte 0.
    End(i64),
    /// Set the offset to the current position plus the specified number of
    /// bytes.
    ///
    /// It is possible to seek beyond the end of an object, but it's an error to
    /// seek before byte 0.
    Current(i64),
}

/// The `Seek` trait provides a cursor which can be moved within a stream of
/// bytes.
pub trait Seek<'sess> {
    /// The error type for I/O operations.
    type Error;

    /// Seek to an offset, in bytes, in a stream.
    ///
    /// A seek beyond the end of a stream is allowed, but implementation
    /// defined.
    ///
    /// If the seek operation completed successfully, this method returns the
    /// new position from the start of the stream. That position can be used
    /// later with [`SeekFrom::Start`](SeekFrom::Start).
    ///
    /// # Errors
    ///
    /// Seeking to a negative offset is considered an error.
    fn seek(
        &'sess mut self,
        pos: SeekFrom,
    ) -> Pin<Box<dyn Future<Output = Result<u64, Self::Error>> + Send + 'sess>>;
}
