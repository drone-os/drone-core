//! *Drone* support code for rustc's built in unit-test and micro-benchmarking
//! framework.


#![feature(core_intrinsics)]
#![feature(lang_items)]

#![no_std]


#[macro_use]
extern crate sc;


pub use TestFn::*;
pub use TestName::*;
use core::{fmt, intrinsics, ptr};


#[macro_use]
pub mod macros;
pub mod io;
pub mod panicking;


static mut TEST_PANICKED: bool = false;


/// Test function with description.
pub struct TestDescAndFn {
  pub desc: TestDesc,
  pub testfn: TestFn,
}


/// The definition of a single test.
pub struct TestDesc {
  pub name: TestName,
  pub ignore: bool,
  pub should_panic: ShouldPanic,
}


/// The name of a test.
pub enum TestName {
  StaticTestName(&'static str),
}


/// A function that runs a test.
pub enum TestFn {
  StaticTestFn(fn()),
}


/// A `should_panic` attribute handler.
#[derive(PartialEq)]
pub enum ShouldPanic {
  No,
  Yes,
}


/// The test runner.
pub fn test_main_static(tests: &[TestDescAndFn]) {
  let mut failed = 0;
  let mut ignored = 0;
  let mut passed = 0;
  eprintln!("running {} tests", tests.len());

  for test in tests {
    let name = match test.desc.name {
      StaticTestName(name) => name,
    };
    let testfn = match test.testfn {
      StaticTestFn(testfn) => testfn,
    };
    if test.desc.ignore {
      ignored += 1;
      eprintln!("test {} ... ignored", name);
    } else {
      eprint!("test {} ... ", name);
      reset_panicked();
      testfn();
      if has_panicked() == (test.desc.should_panic == ShouldPanic::Yes) {
        passed += 1;
        eprintln!("OK");
      } else {
        failed += 1;
        eprintln!("FAILED");
      }
    }
  }

  eprintln!();
  eprintln!(
    "test result: {}. {} passed; {} failed; {} ignored",
    if failed == 0 { "OK" } else { "FAILED" },
    passed,
    failed,
    ignored,
  );

  if failed != 0 {
    exit(101);
  }
}


/// Overridden panic routine.
pub fn test_panic(args: fmt::Arguments, file: &'static str, line: u32) {
  unsafe {
    TEST_PANICKED = true;
  }
  eprintln!();
  eprint!("panicked at '");
  io::write_fmt(args);
  eprintln!("', {}:{}", file, line);
}


/// Entry point.
#[no_mangle]
pub unsafe extern "C" fn _start() -> ! {
  extern "C" {
    fn main(argc: isize, argv: *const *const u8) -> isize;
  }
  main(0, ptr::null());
  exit(0);
}


/// Lang item required to run `main`.
#[lang = "start"]
extern "C" fn start(
  main: fn(),
  _argc: isize,
  _argv: *const *const u8,
) -> isize {
  main();
  0
}


#[doc(hidden)]
#[no_mangle]
pub extern "C" fn __aeabi_unwind_cpp_pr0() {}


#[doc(hidden)]
#[no_mangle]
pub extern "C" fn __aeabi_unwind_cpp_pr1() {}


fn reset_panicked() {
  unsafe {
    TEST_PANICKED = false;
  }
}


fn has_panicked() -> bool {
  unsafe { TEST_PANICKED }
}


fn exit(code: i32) -> ! {
  unsafe {
    syscall!(EXIT, code as usize);
    intrinsics::unreachable()
  }
}
