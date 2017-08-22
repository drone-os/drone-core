//! Panicking support.


use {itm, util};
use core::fmt;


/// Panic handler.
///
/// It attempts to write a panic message to ITM and resets the device.
#[cfg_attr(feature = "cargo-clippy", allow(empty_loop))]
#[linkage = "weak"]
#[lang = "panic_fmt"]
extern "C" fn begin(args: fmt::Arguments, file: &'static str, line: u32) -> ! {
  iprint!("panicked at '");
  itm::write_fmt(args);
  iprintln!("', {}:{}", file, line);
  itm::flush();
  util::reset_request();
  loop {}
}
