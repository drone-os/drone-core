//! Drone Stream.
//!
//! This module implements standard output/error interface, which mimics Rust's
//! standard library.

mod macros;
mod runtime;

use self::runtime::Runtime;
use core::{fmt, fmt::Write};

#[doc(hidden)]
#[link_section = ".stream_rt"]
#[no_mangle]
#[used]
static RT: Runtime = Runtime::new();

/// Maximum number of streams.
pub const STREAM_COUNT: u8 = 32;

/// Stream number of the standard output.
pub const STDOUT_STREAM: u8 = 0;

/// Stream number of the standard error.
pub const STDERR_STREAM: u8 = 1;

/// Stream handle.
#[derive(Clone, Copy)]
pub struct Stream(u8);

/// Returns a stream for the standard output.
#[inline]
pub fn stdout() -> Stream {
    Stream::new(STDOUT_STREAM)
}

/// Returns a stream for the standard error.
#[inline]
pub fn stderr() -> Stream {
    Stream::new(STDERR_STREAM)
}

/// Writes some data into a specific stream.
///
/// This function doesn't check whether the stream is enabled by a debug
/// probe. It's recommended to use this function in conjunction with
/// [`Stream::is_enabled`].
///
/// # Examples
///
/// ```
/// use drone_core::{stream, stream::Stream};
///
/// if Stream::new(11).is_enabled() {
///     stream::write_str(11, "hello there!\n");
/// }
/// ```
#[inline(never)]
pub fn write_str(stream: u8, value: &str) {
    let _ = Stream::new(stream).write_str(value);
}

/// Writes some formatted information into a specific stream.
///
/// This function doesn't check whether the stream is enabled by a debug
/// probe. It's recommended to use this function in conjunction with
/// [`Stream::is_enabled`].
///
/// # Examples
///
/// ```
/// use drone_core::{stream, stream::Stream};
///
/// let a = 0;
///
/// if Stream::new(11).is_enabled() {
///     stream::write_fmt(11, format_args!("a = {}\n", a));
/// }
/// ```
#[inline(never)]
pub fn write_fmt(stream: u8, args: fmt::Arguments<'_>) {
    let _ = Stream::new(stream).write_fmt(args);
}

impl Stream {
    /// Creates a new stream handle.
    ///
    /// # Panics
    ///
    /// If `stream` is more than or equal to [`STREAM_COUNT`].
    #[inline]
    pub fn new(stream: u8) -> Self {
        assert!(stream < STREAM_COUNT);
        Self(stream)
    }

    /// Returns `true` if this stream is explicitly enabled by a debug probe in
    /// the run-time, returns `false` by default.
    #[inline]
    pub fn is_enabled(self) -> bool {
        let Self(stream) = self;
        RT.is_enabled(stream)
    }

    /// Writes a sequence of bytes to this stream.
    ///
    /// The resulting byte sequence visible to a debug probe may be interleaved
    /// with other concurrent writes. See also [`Stream::write`] for writing
    /// atomic byte sequences.
    #[inline]
    #[allow(clippy::return_self_not_must_use)]
    pub fn write_bytes(self, bytes: &[u8]) -> Self {
        let Self(stream) = self;
        RT.write_bytes(stream, bytes.as_ptr(), bytes.len());
        self
    }

    /// Writes an atomic byte sequence to this stream. `T` can be one of `u8`,
    /// `u16`, `u32`.
    ///
    /// Bytes are written in big-endian order. It's guaranteed that all bytes
    /// of `value` will be visible be a debug probe indissolubly.
    #[inline]
    #[allow(clippy::return_self_not_must_use)]
    pub fn write<T: sealed::StreamWrite>(self, value: T) -> Self {
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

mod sealed {
    pub trait StreamWrite: Copy {
        fn stream_write(stream: u8, value: Self);
    }

    impl StreamWrite for u8 {
        fn stream_write(stream: u8, value: Self) {
            super::RT.write_u8(stream, value);
        }
    }

    impl StreamWrite for u16 {
        fn stream_write(stream: u8, value: Self) {
            super::RT.write_u16(stream, value);
        }
    }

    impl StreamWrite for u32 {
        fn stream_write(stream: u8, value: Self) {
            super::RT.write_u32(stream, value);
        }
    }
}
