//! Drone Stream.
//!
//! This module implements standard output/error interface, which mimics Rust's
//! standard library.

mod macros;
mod runtime;

pub use drone_stream::STREAM_COUNT;

use self::runtime::LocalRuntime;
use core::{
    cell::{SyncUnsafeCell, UnsafeCell},
    fmt,
    fmt::Write,
    mem, ptr,
};
use drone_stream::{Runtime, BOOTSTRAP_SEQUENCE, BOOTSTRAP_SEQUENCE_LENGTH};

extern "C" {
    static STREAM_START: UnsafeCell<u8>;
    static STREAM_END: UnsafeCell<u8>;
}

#[doc(hidden)]
#[link_section = ".stream_rt"]
#[no_mangle]
#[used]
static RT: SyncUnsafeCell<Runtime> = SyncUnsafeCell::new(Runtime::zeroed());

/// Stream number of the standard output.
pub const STDOUT_STREAM: u8 = 0;

/// Stream number of the standard error.
pub const STDERR_STREAM: u8 = 1;

/// Stream handle.
#[derive(Clone, Copy)]
pub struct Stream(u8);

/// Initializes the Drone Stream runtime.
pub fn init() {
    unsafe {
        // Check if the debug probe wants to modify the runtime structure as
        // soon as possible.
        let mut buffer = STREAM_START.get();
        let mut sample = BOOTSTRAP_SEQUENCE.as_ptr();
        let mut counter = BOOTSTRAP_SEQUENCE_LENGTH;
        while counter > 0 && *buffer == *sample {
            buffer = buffer.add(1);
            sample = sample.add(1);
            counter -= 1;
        }
        if counter == 0 {
            // Found the valid bootstrap sequence. Copy the bytes, which follow
            // it, into the runtime structure.
            ptr::copy_nonoverlapping(
                buffer,
                STREAM_START.get().sub(mem::size_of::<Runtime>()),
                mem::size_of::<Runtime>(),
            );
            // Invalidate the bootstrap sequence.
            *STREAM_START.get() = 0;
        }
    }
}

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
        rt().is_enabled(stream)
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
        rt().write_bytes(stream, bytes.as_ptr(), bytes.len());
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

fn rt() -> &'static Runtime {
    unsafe { &*RT.get().as_const() }
}

mod sealed {
    use super::{rt, LocalRuntime};

    pub trait StreamWrite: Copy {
        fn stream_write(stream: u8, value: Self);
    }

    impl StreamWrite for u8 {
        fn stream_write(stream: u8, value: Self) {
            rt().write_u8(stream, value);
        }
    }

    impl StreamWrite for u16 {
        fn stream_write(stream: u8, value: Self) {
            rt().write_u16(stream, value);
        }
    }

    impl StreamWrite for u32 {
        fn stream_write(stream: u8, value: Self) {
            rt().write_u32(stream, value);
        }
    }
}
