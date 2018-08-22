//! Platform-specific types, as defined by C.
//!
//! Part of [Drone].
//!
//! [Drone]: https://github.com/drone-os/drone-core

#![allow(non_camel_case_types)]
#![warn(missing_docs)]
#![doc(html_root_url = "https://docs.rs/drone-ctypes/0.8.3")]
#![no_std]
#![cfg_attr(feature = "cargo-clippy", allow(precedence))]

use core::fmt;

/// Equivalent to C's `char` type.
pub type c_char = u8;

/// Equivalent to C's `signed char` type.
pub type c_schar = i8;

/// Equivalent to C's `unsigned char` type.
pub type c_uchar = u8;

/// Equivalent to C's `signed short` (`short`) type.
pub type c_short = i16;

/// Equivalent to C's `unsigned short` type.
pub type c_ushort = u16;

/// Equivalent to C's `signed int` (`int`) type.
pub type c_int = i32;

/// Equivalent to C's `unsigned int` type.
pub type c_uint = u32;

/// Equivalent to C's `signed long` (`long`) type.
#[cfg(target_pointer_width = "32")]
pub type c_long = i32;

/// Equivalent to C's `unsigned long` type.
#[cfg(target_pointer_width = "32")]
pub type c_ulong = u32;

/// Equivalent to C's `signed long` (`long`) type.
#[cfg(target_pointer_width = "64")]
pub type c_long = i64;

/// Equivalent to C's `unsigned long` type.
#[cfg(target_pointer_width = "64")]
pub type c_ulong = u64;

/// Equivalent to C's `signed long long` (`long long`) type.
pub type c_longlong = i64;

/// Equivalent to C's `unsigned long long` type.
pub type c_ulonglong = u64;

/// Equivalent to C's `float` type.
pub type c_float = f32;

/// Equivalent to C's `double` type.
pub type c_double = f64;

/// Equivalent to C's `void` type when used as a pointer.
#[repr(u8)]
pub enum c_void {
  #[doc(hidden)]
  __variant,
}

impl fmt::Debug for c_void {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.pad("c_void")
  }
}
