use core::future::Future;
use core::pin::Pin;

/// The `Write` trait allows for writing bytes to a source asynchronously.
pub trait Write<'sess> {
    /// The error type returned by [`Write::write`].
    type Error;

    /// Write a buffer into this writer asynchronously, eventually returning how
    /// many bytes were written.
    fn write(
        &'sess mut self,
        buf: &'sess [u8],
    ) -> Pin<Box<dyn Future<Output = Result<usize, Self::Error>> + Send + 'sess>>;
}
