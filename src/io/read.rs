use core::{future::Future, pin::Pin};

/// The `Read` trait allows for reading bytes from a source asynchronously.
pub trait Read<'sess> {
    /// The error type returned by [`Read::read`].
    type Error;

    /// Pull some bytes from this source into the specified buffer
    /// asynchronously, eventually returning how many bytes were read.
    fn read(
        &'sess mut self,
        buf: &'sess mut [u8],
    ) -> Pin<Box<dyn Future<Output = Result<usize, Self::Error>> + Send + 'sess>>;
}
