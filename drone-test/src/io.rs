//! IO from/into host OS.


use core::fmt::{self, Write};
use core::slice;


const STDERR: usize = 2;


struct Stderr;


impl Stderr {
  fn write_all(&mut self, mut buffer: &[u8]) {
    while !buffer.is_empty() {
      let result = unsafe {
        syscall!(WRITE, STDERR, buffer.as_ptr(), buffer.len()) as isize
      };
      if result < 0 {
        return;
      }
      buffer = unsafe {
        slice::from_raw_parts(
          buffer.as_ptr().offset(result),
          buffer.len() - result as usize,
        )
      }
    }
  }
}


impl Write for Stderr {
  fn write_str(&mut self, string: &str) -> fmt::Result {
    self.write_all(string.as_bytes());
    Ok(())
  }
}


/// Prints `str` to the host OS.
///
/// See [eprint](../macro.eprint.html) and [eprintln](../macro.eprintln.html)
/// macros.
pub fn write_str(string: &str) {
  Stderr.write_all(string.as_bytes())
}


/// Prints `core::fmt::Arguments` to the host OS.
///
/// See [eprint](../macro.eprint.html) and [eprintln](../macro.eprintln.html)
/// macros.
pub fn write_fmt(args: fmt::Arguments) {
  Stderr.write_fmt(args).unwrap();
}
