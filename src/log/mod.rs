//! Debug logging facade.
//!
//! This module implements standard output/error interface, which mimics Rust's
//! standard library. This is a facade module. Concrete output implementation
//! should be provided by downstream crates.
//!
//! Reserved ports:
//!
//! * `0` - standard output
//! * `1` - standard error
//! * `31` - heap trace

#![cfg_attr(feature = "std", allow(unreachable_code, unused_variables))]

mod macros;
mod port;

/// Returns log output baud rate defined in `Drone.toml`.
///
/// # Examples
///
/// ```
/// # #![feature(proc_macro_hygiene)]
/// # drone_core::config_override! { "
/// # [memory]
/// # flash = { size = \"128K\", origin = 0x08000000 }
/// # ram = { size = \"20K\", origin = 0x20000000 }
/// # [heap]
/// # size = \"0\"
/// # pools = []
/// # [probe]
/// # gdb-client-command = \"gdb-multiarch\"
/// # [probe.dso]
/// # baud-rate = 115200
/// # serial-endpoint = \"/dev/ttyACM0\"
/// # " }
/// use drone_core::log;
///
/// assert_eq!(log::baud_rate!(), 115_200);
/// ```
#[doc(inline)]
pub use drone_core_macros::log_baud_rate as baud_rate;

pub use self::port::Port;

use core::{fmt, fmt::Write};

extern "C" {
    pub fn drone_log_is_enabled(port: u8) -> bool;
    pub fn drone_log_write_bytes(port: u8, buffer: *const u8, count: usize);
    pub fn drone_log_write_u8(port: u8, value: u8);
    pub fn drone_log_write_u16(port: u8, value: u16);
    pub fn drone_log_write_u32(port: u8, value: u32);
    pub fn drone_log_flush();
}

/// Number of ports.
pub const PORTS_COUNT: u8 = 32;

/// Port number of the standard output stream.
pub const STDOUT_PORT: u8 = 0;

/// Port number of the standard error stream.
pub const STDERR_PORT: u8 = 1;

/// Port number of the heap trace stream.
pub const HEAPTRACE_PORT: u8 = 31;

/// Returns port for standard output.
#[inline]
pub fn stdout() -> Port {
    Port::new(STDOUT_PORT)
}

/// Returns port for standard error.
#[inline]
pub fn stderr() -> Port {
    Port::new(STDERR_PORT)
}

/// Writes `string` to the log port number `port`.
///
/// The presence of the debug probe is not checked, so it is recommended to use
/// this function together with [`Port::is_enabled`].
///
/// # Examples
///
/// ```
/// use drone_core::{log, log::Port};
///
/// if Port::new(11).is_enabled() {
///     log::write_str(11, "hello there!\n");
/// }
/// ```
#[inline(never)]
pub fn write_str(port: u8, string: &str) {
    let _ = Port::new(port).write_str(string);
}

/// Writes `args` to the log port number `port`.
///
/// The presence of the debug probe is not checked, so it is recommended to use
/// this function together with [`Port::is_enabled`].
///
/// # Examples
///
/// ```
/// use drone_core::{log, log::Port};
///
/// let a = 0;
///
/// if Port::new(11).is_enabled() {
///     log::write_fmt(11, format_args!("a = {}\n", a));
/// }
/// ```
#[inline(never)]
pub fn write_fmt(port: u8, args: fmt::Arguments<'_>) {
    let _ = Port::new(port).write_fmt(args);
}

/// Blocks until all pending packets are transmitted.
///
/// This function is a no-op if no debug probe is connected and listening.
#[inline]
pub fn flush() {
    #[cfg(feature = "std")]
    return;
    unsafe { drone_log_flush() };
}
