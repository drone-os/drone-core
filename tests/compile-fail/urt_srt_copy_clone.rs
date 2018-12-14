#[macro_use]
extern crate drone_core;

use drone_core::reg;
use drone_core::reg::prelude::*;

reg!(pub mod FOO BAR; 0xDEAD_BEEF 0x20 0xBEEF_CACE; BAZ { 0 1 });

fn assert_copy<T: Copy>() {}
fn assert_clone<T: Clone>() {}

fn main() {
  assert_copy::<foo_bar::Reg<Urt>>();
  //~^ ERROR `drone_core::reg::Urt: std::marker::Copy` is not satisfied
  assert_clone::<foo_bar::Reg<Urt>>();
  //~^ ERROR `drone_core::reg::Urt: std::clone::Clone` is not satisfied
  assert_copy::<foo_bar::Reg<Srt>>();
  //~^ ERROR `drone_core::reg::Srt: std::marker::Copy` is not satisfied
  assert_clone::<foo_bar::Reg<Srt>>();
  //~^ ERROR `drone_core::reg::Srt: std::clone::Clone` is not satisfied
}
