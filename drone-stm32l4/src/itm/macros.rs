//! Collection of macros.

/// Macro for printing through ITM.
#[macro_export]
macro_rules! iprint {
  ($str:expr) => {
    $crate::itm::write_str($str);
  };

  ($($arg:tt)*) => {
    $crate::itm::write_fmt(format_args!($($arg)*));
  };
}

/// Macro for printing through ITM, with a newline.
#[macro_export]
macro_rules! iprintln {
  () => {
    iprint!("\n");
  };

  ($fmt:expr) => {
    iprint!(concat!($fmt, "\n"));
  };

  ($fmt:expr, $($arg:tt)*) => {
    iprint!(concat!($fmt, "\n"), $($arg)*);
  };
}
