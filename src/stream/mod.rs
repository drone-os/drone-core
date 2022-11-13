//! Drone Stream.
//!
//! This module implements standard output/error interface, which mimics Rust's
//! standard library.

#![cfg_attr(feature = "host", allow(unused_imports, dead_code, unreachable_code, unused_variables))]

mod macros;
mod runtime;

use self::runtime::{LocalGlobalRuntime, LocalRuntime};
use crate::platform::stream_rt;
use core::cell::SyncUnsafeCell;
use core::fmt::Write;
use core::mem::size_of;
use core::{fmt, mem, ptr};
pub use drone_stream::STREAM_COUNT;
use drone_stream::{GlobalRuntime, Runtime, BOOTSTRAP_SEQUENCE, BOOTSTRAP_SEQUENCE_LENGTH};

#[link_section = ".stream_rt"]
#[no_mangle]
static GLOBAL_RT: SyncUnsafeCell<GlobalRuntime> = SyncUnsafeCell::new(GlobalRuntime::zeroed());

/// Stream number of the standard output.
pub const STDOUT_STREAM: u8 = 0;

/// Stream number of the standard error.
pub const STDERR_STREAM: u8 = 1;

/// Stream handle.
#[derive(Clone, Copy)]
pub struct Stream(u8);

#[doc(hidden)]
#[inline(never)]
pub unsafe fn init(rt: *mut Runtime, buffer_size: u32, init_global: bool) {
    #[cfg(feature = "host")]
    return unimplemented!();
    #[cfg(not(feature = "host"))]
    unsafe {
        // Check if the debug probe wants to modify the runtime structure as
        // soon as possible.
        let mut buffer = rt.add(1).cast::<u8>();
        let mut sample = BOOTSTRAP_SEQUENCE.as_ptr();
        let mut counter = BOOTSTRAP_SEQUENCE_LENGTH;
        while counter > 0 && *buffer == *sample {
            buffer = buffer.add(1);
            sample = sample.add(1);
            counter -= 1;
        }
        if counter == 0 {
            // Found the valid bootstrap sequence. Copy the bytes, which follow
            // it, into the runtime structures.
            ptr::copy_nonoverlapping(buffer, rt.cast::<u8>(), mem::size_of::<Runtime>());
            buffer = buffer.add(mem::size_of::<Runtime>());
            if init_global {
                ptr::copy_nonoverlapping(
                    buffer,
                    GLOBAL_RT.get().cast(),
                    mem::size_of::<GlobalRuntime>(),
                );
            }
            // Invalidate the bootstrap sequence.
            *rt.add(1).cast::<u8>() = 0;
        } else {
            if init_global {
                ptr::write_bytes(GLOBAL_RT.get().cast::<u8>(), 0, size_of::<GlobalRuntime>());
            }
            *rt = Runtime { buffer_size, read_cursor: 0, write_cursor: 0 };
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
/// use drone_core::stream;
/// use drone_core::stream::Stream;
///
/// if Stream::new(11).is_enabled() {
///     stream::write_str(11, "hello there!\n");
/// }
/// ```
#[inline(never)]
#[export_name = "stream_write_str"]
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
/// use drone_core::stream;
/// use drone_core::stream::Stream;
///
/// let a = 0;
///
/// if Stream::new(11).is_enabled() {
///     stream::write_fmt(11, format_args!("a = {}\n", a));
/// }
/// ```
#[inline(never)]
#[export_name = "stream_write_fmt"]
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
        unsafe { (*GLOBAL_RT.get()).is_enabled(stream) }
    }

    /// Writes a sequence of bytes to this stream.
    ///
    /// The resulting byte sequence visible to a debug probe may be interleaved
    /// with other concurrent writes. See also [`Stream::write`] for writing
    /// atomic byte sequences.
    #[allow(clippy::return_self_not_must_use)]
    #[inline]
    pub fn write_bytes(self, bytes: &[u8]) -> Self {
        let Self(stream) = self;
        unsafe { (*stream_rt()).write_bytes(stream, bytes.as_ptr(), bytes.len()) };
        self
    }

    /// Writes a sequence of bytes to this stream in one transaction.
    ///
    /// # Panics
    ///
    /// If length of `bytes` is more than 256.
    #[allow(clippy::return_self_not_must_use)]
    #[inline]
    pub fn write_transaction(self, bytes: &[u8]) -> Self {
        let Self(stream) = self;
        let length = bytes.len().try_into().expect("maximum transaction length exceeded");
        unsafe { (*stream_rt()).write_transaction(stream, bytes.as_ptr(), length) };
        self
    }

    /// Writes `T` as a sequence of bytes to this stream in one transaction.
    /// `T` can be one of  `u8`, `u16`, `u32`.
    #[allow(clippy::return_self_not_must_use)]
    #[inline]
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
    use super::{stream_rt, LocalRuntime};

    pub trait StreamWrite: Copy {
        fn stream_write(stream: u8, value: Self);
    }

    macro_rules! impl_integer {
        ($ty:ty) => {
            impl StreamWrite for $ty {
                #[inline]
                fn stream_write(stream: u8, value: Self) {
                    let bytes = value.to_ne_bytes();
                    unsafe {
                        (*stream_rt()).write_transaction(stream, bytes.as_ptr(), bytes.len() as u8);
                    }
                }
            }
        };
    }

    impl_integer!(u8);
    impl_integer!(u16);
    impl_integer!(u32);
}
