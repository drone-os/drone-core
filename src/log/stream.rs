use super::{CONTROL, STREAMS_COUNT};
use core::{fmt, fmt::Write};

/// Logging stream handle.
#[derive(Clone, Copy)]
pub struct Stream(u8);

pub trait StreamWrite: Copy {
    fn stream_write(stream: u8, value: Self);
}

impl Stream {
    /// Creates a new stream handle.
    ///
    /// # Panics
    ///
    /// If `stream` is more than or equal to [`STREAMS_COUNT`].
    #[inline]
    pub fn new(stream: u8) -> Self {
        assert!(stream < STREAMS_COUNT);
        Self(stream)
    }

    /// Returns `true` if the debug probe is listening to the stream.
    #[inline]
    pub fn is_enabled(self) -> bool {
        let Self(stream) = self;
        CONTROL.is_enabled(stream)
    }

    /// Writes a sequence of bytes to the stream.
    ///
    /// The resulting byte sequence that will be read from the stream may be
    /// interleaved with concurrent writes. See also [`Stream::write`] for
    /// writing atomic byte sequences.
    #[inline]
    pub fn write_bytes(self, bytes: &[u8]) -> Self {
        let Self(stream) = self;
        CONTROL.write_bytes(stream, bytes.as_ptr(), bytes.len());
        self
    }

    /// Writes an atomic byte sequence to the stream. `T` can be one of `u8`,
    /// `u16`, `u32`.
    ///
    /// Bytes are written in big-endian order. It's guaranteed that all bytes of
    /// `value` will not be split.
    #[inline]
    pub fn write<T: StreamWrite>(self, value: T) -> Self {
        let Self(stream) = self;
        T::stream_write(stream, value);
        self
    }
}

impl Write for Stream {
    #[inline]
    fn write_str(&mut self, string: &str) -> fmt::Result {
        self.write_bytes(string.as_bytes());
        Ok(())
    }
}

impl StreamWrite for u8 {
    fn stream_write(stream: u8, value: Self) {
        CONTROL.write_u8(stream, value);
    }
}

impl StreamWrite for u16 {
    fn stream_write(stream: u8, value: Self) {
        CONTROL.write_u16(stream, value);
    }
}

impl StreamWrite for u32 {
    fn stream_write(stream: u8, value: Self) {
        CONTROL.write_u32(stream, value);
    }
}
