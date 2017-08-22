//! Panicking support.


use core::{fmt, intrinsics};
use io;


/// Panic handler.
#[cfg(feature = "panic_item")]
#[lang = "panic_fmt"]
extern "C" fn panic_fmt(
  args: fmt::Arguments,
  file: &'static str,
  line: u32,
) -> ! {
  panic_handler(args, file, line)
}


/// Overridden panic handler.
#[cfg(not(feature = "panic_item"))]
#[no_mangle]
pub extern "C" fn rust_begin_unwind(
  args: fmt::Arguments,
  file: &'static str,
  line: u32,
) -> ! {
  panic_handler(args, file, line)
}


fn panic_handler(args: fmt::Arguments, file: &'static str, line: u32) -> ! {
  eprintln!();
  eprint!("Uncaught panic at '");
  io::write_fmt(args);
  eprintln!("', {}:{}", file, line);
  eprintln!("ABORT");
  unsafe { intrinsics::abort() }
}
