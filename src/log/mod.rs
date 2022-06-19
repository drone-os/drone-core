//! Drone Logging.
//!
//! This module implements standard output/error interface, which mimics Rust's
//! standard library.

mod control;
mod macros;
mod stream;

pub use self::stream::Stream;

use self::control::Control;
use core::{fmt, fmt::Write};

#[doc(hidden)]
#[link_section = ".log"]
#[no_mangle]
#[used]
static CTRL: Control = Control::new();

/// Number of streams.
pub const STREAMS_COUNT: u8 = 32;

/// Stream number of the standard output.
pub const STDOUT_NUMBER: u8 = 0;

/// Stream number of the standard error.
pub const STDERR_NUMBER: u8 = 1;

/// Returns a stream for the standard output.
#[inline]
pub fn stdout() -> Stream {
    Stream::new(STDOUT_NUMBER)
}

/// Returns a stream for the standard error.
#[inline]
pub fn stderr() -> Stream {
    Stream::new(STDERR_NUMBER)
}

/// Writes the string `value` to the stream number `stream`.
///
/// This function doesn't check whether the logging is enabled by the debug
/// probe. It's recommended to use this function together with
/// [`Stream::is_enabled`].
///
/// # Examples
///
/// ```
/// use drone_core::{log, log::Stream};
///
/// if Stream::new(11).is_enabled() {
///     log::write_str(11, "hello there!\n");
/// }
/// ```
#[inline(never)]
pub fn write_str(stream: u8, value: &str) {
    let _ = Stream::new(stream).write_str(value);
}

/// Writes the formatting `args` to the log stream number `stream`.
///
/// This function doesn't check whether the logging is enabled by the debug
/// probe. It's recommended to use this function together with
/// [`Stream::is_enabled`].
///
/// # Examples
///
/// ```
/// use drone_core::{log, log::Stream};
///
/// let a = 0;
///
/// if Stream::new(11).is_enabled() {
///     log::write_fmt(11, format_args!("a = {}\n", a));
/// }
/// ```
#[inline(never)]
pub fn write_fmt(stream: u8, args: fmt::Arguments<'_>) {
    let _ = Stream::new(stream).write_fmt(args);
}
