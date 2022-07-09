/// Prints to the standard output (stream number 0).
///
/// This is almost a no-op until a debug probe explicitly enables the
/// corresponding stream in the run-time.
///
/// Equivalent to the [`println!`] macro except that a newline is not printed
/// at the end of the message.
///
/// Use `print!` only for the primary output of your program. Use [`eprint!`]
/// instead to print error and progress messages.
///
/// # Examples
///
/// ```
/// use drone_core::{print, stream};
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
        if $crate::stream::stdout().is_enabled() {
            $crate::stream::write_str($crate::stream::STDOUT_STREAM, $str);
        }
    };
    ($($arg:tt)*) => {
        if $crate::stream::stdout().is_enabled() {
            $crate::stream::write_fmt(
                $crate::stream::STDOUT_STREAM,
                $crate::_rt::core::format_args!($($arg)*),
            );
        }
    };
}

/// Prints to the standard output (stream number 0), with a newline.
///
/// This macro uses the same syntax as [`format!`], but writes to the standard
/// output instead. See [`core::fmt`] for more information.
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
        $crate::print!("\n")
    };
    ($str:expr) => {
        $crate::print!($crate::_rt::core::concat!($str, "\n"))
    };
    ($str:expr, $($arg:tt)*) => {
        $crate::print!($crate::_rt::core::concat!($str, "\n"), $($arg)*)
    };
}

/// Prints to the standard error (stream number 1).
///
/// Equivalent to the [`print!`] macro, except that output goes to the stream
/// #1 instead of #0. See [`print!`] for example usage.
///
/// Use `eprint!` only for error and progress messages. Use `print!` instead
/// for the primary output of your program.
///
/// # Examples
///
/// ```
/// use drone_core::eprint;
///
/// eprint!("Error: Could not complete task");
/// ```
#[macro_export]
macro_rules! eprint {
    ($str:expr) => {
        if $crate::stream::stderr().is_enabled() {
            $crate::stream::write_str($crate::stream::STDERR_STREAM, $str);
        }
    };
    ($($arg:tt)*) => {
        if $crate::stream::stderr().is_enabled() {
            $crate::stream::write_fmt(
                $crate::stream::STDERR_STREAM,
                $crate::_rt::core::format_args!($($arg)*),
            );
        }
    };
}

/// Prints to the standard error (stream number 1), with a newline.
///
/// Equivalent to the [`println!`] macro, except that output goes to the stream
/// #1 instead of #0. See [`println!`] for example usage.
///
/// Use `eprintln!` only for error and progress messages. Use `println!`
/// instead for the primary output of your program.
///
/// # Examples
///
/// ```
/// use drone_core::eprintln;
///
/// eprintln!("Error: Could not complete task");
/// ```
#[macro_export]
macro_rules! eprintln {
    () => {
        $crate::eprint!("\n")
    };
    ($str:expr) => {
        $crate::eprint!($crate::_rt::core::concat!($str, "\n"))
    };
    ($str:expr, $($arg:tt)*) => {
        $crate::eprint!($crate::_rt::core::concat!($str, "\n"), $($arg)*)
    };
}

/// Prints and returns the value of a given expression for quick and dirty
/// debugging.
///
/// The macro works by using the `Debug` implementation of the type of the
/// given expression to print the value to the stream #1 along with the
/// source location of the macro invocation as well as the source code of the
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
/// use drone_core::dbg;
///
/// let a = 2;
/// let b = dbg!(a * 2) + 1;
/// //      ^-- prints: [src/main.rs:2] a * 2 = 4
/// assert_eq!(b, 5);
/// ```
#[macro_export]
macro_rules! dbg {
    () => {
        $crate::eprintln!(
            "[{}:{}]",
            $crate::_rt::core::file!(),
            $crate::_rt::core::line!(),
        )
    };
    ($val:expr $(,)?) => {
        match $val {
            tmp => {
                $crate::eprintln!(
                    "[{}:{}] {} = {:#?}",
                    $crate::_rt::core::file!(),
                    $crate::_rt::core::line!(),
                    $crate::_rt::core::stringify!($val),
                    &tmp,
                );
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($($crate::dbg!($val)),+,)
    };
}
