/// Macro for printing to the standard error of host OS.
macro_rules! eprint {
  ($str:expr) => {
    $crate::io::write_str($str);
  };

  ($($arg:tt)*) => {
    $crate::io::write_fmt(format_args!($($arg)*));
  };
}

/// Macro for printing to the standard error of host OS, with a newline.
macro_rules! eprintln {
  () => {
    eprint!("\n");
  };

  ($fmt:expr) => {
    eprint!(concat!($fmt, "\n"));
  };

  ($fmt:expr, $($arg:tt)*) => {
    eprint!(concat!($fmt, "\n"), $($arg)*);
  };
}

/// Override of the standard `panic!` macro.
#[macro_export]
macro_rules! panic {
  () => {
    panic!("explicit panic");
  };

  ($fmt:expr) => {
    panic!($fmt,);
  };

  ($fmt:expr, $($arg:tt)*) => {
    $crate::test_panic(format_args!($fmt, $($arg)*), file!(), line!());
  };
}
