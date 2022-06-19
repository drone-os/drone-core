/// Prints to the log stream #0, if logging is enabled in the run-time by the
/// debug probe.
///
/// Equivalent to the [`println!`] macro except that a newline is not printed at
/// the end of the message.
///
/// Use `print!` only for the primary output of your program. Use [`eprint!`]
/// instead to print error and progress messages.
///
/// # Examples
///
/// ```
/// use drone_core::{log, print};
///
/// print!("this ");
/// print!("will ");
/// print!("be ");
/// print!("on ");
/// print!("the ");
/// print!("same ");
/// print!("line ");
///
/// print!("this string has a newline, why not choose println! instead?\n");
/// ```
#[macro_export]
macro_rules! print {
    ($str:expr) => {
        if $crate::log::stdout().is_enabled() {
            $crate::log::write_str($crate::log::STDOUT_NUMBER, $str);
        }
    };
    ($($arg:tt)*) => {
        if $crate::log::stdout().is_enabled() {
            $crate::log::write_fmt($crate::log::STDOUT_NUMBER, format_args!($($arg)*));
        }
    };
}

/// Prints to the log stream #0, with a newline, if logging is enabled in the
/// run-time by the debug probe.
///
/// Use the `format!` syntax to write data to the standard output. See
/// [`core::fmt`] for more information.
///
/// Use `println!` only for the primary output of your program. Use
/// [`eprintln!`] instead to print error and progress messages.
///
/// # Examples
///
/// ```
/// use drone_core::println;
///
/// println!(); // prints just a newline
/// println!("hello there!");
/// println!("format {} arguments", "some");
/// ```
#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\n");
    };
    ($fmt:expr) => {
        $crate::print!(concat!($fmt, "\n"));
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::print!(concat!($fmt, "\n"), $($arg)*);
    };
}

/// Prints to the log stream #1, if logging is enabled in the run-time by the
/// debug probe.
///
/// Equivalent to the [`print!`] macro, except that output goes to the stream #1
/// instead of #0. See [`print!`] for example usage.
///
/// Use `eprint!` only for error and progress messages. Use `print!` instead for
/// the primary output of your program.
///
/// # Examples
///
/// ```
/// eprint!("Error: Could not complete task");
/// ```
#[macro_export]
macro_rules! eprint {
    ($str:expr) => {
        if $crate::log::stderr().is_enabled() {
            $crate::log::write_str($crate::log::STDERR_NUMBER, $str);
        }
    };
    ($($arg:tt)*) => {
        if $crate::log::stderr().is_enabled() {
            $crate::log::write_fmt($crate::log::STDERR_NUMBER, format_args!($($arg)*));
        }
    };
}

/// Prints to the log stream #1, with a newline, if logging is enabled in the
/// run-time by the debug probe.
///
/// Equivalent to the [`println!`] macro, except that output goes to the stream
/// #1 instead of #0. See [`println!`] for example usage.
///
/// Use `eprintln!` only for error and progress messages. Use `println!` instead
/// for the primary output of your program.
///
/// # Examples
///
/// ```
/// eprintln!("Error: Could not complete task");
/// ```
#[macro_export]
macro_rules! eprintln {
    () => {
        $crate::eprint!("\n");
    };
    ($fmt:expr) => {
        $crate::eprint!(concat!($fmt, "\n"));
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::eprint!(concat!($fmt, "\n"), $($arg)*);
    };
}

/// Prints and returns the value of a given expression for quick and dirty
/// debugging.
///
/// The macro works by using the `Debug` implementation of the type of the given
/// expression to print the value to the log stream #1 along with the source
/// location of the macro invocation as well as the source code of the
/// expression.
///
/// Invoking the macro on an expression moves and takes ownership of it before
/// returning the evaluated expression unchanged. If the type of the expression
/// does not implement `Copy` and you don't want to give up ownership, you can
/// instead borrow with `dbg!(&expr)` for some expression `expr`.
///
/// # Examples
///
/// ```
/// let a = 2;
/// let b = dbg!(a * 2) + 1;
/// //      ^-- prints: [src/main.rs:2] a * 2 = 4
/// assert_eq!(b, 5);
/// ```
#[macro_export]
macro_rules! dbg {
    () => {
        $crate::eprintln!("[{}:{}]", file!(), line!());
    };
    ($val:expr) => {
        match $val {
            tmp => {
                $crate::eprintln!("[{}:{}] {} = {:#?}", file!(), line!(), stringify!($val), &tmp);
                tmp
            }
        }
    };
    ($val:expr,) => { $crate::dbg!($val) };
    ($($val:expr),+ $(,)?) => {
        ($($crate::dbg!($val)),+,)
    };
}
