#![feature(proc_macro)]

extern crate drone_core;

use drone_core::reg::mappings;
use drone_core::reg::prelude::*;

mappings!(FOO; BAR { 0xDEAD_BEEF 0x20 0xBEEF_CACE; BAZ { 0 1 } });

fn assert_copy<T: Copy>() {}
fn assert_clone<T: Clone>() {}

fn main() {
  assert_copy::<foo::Bar<Urt>>();
  //~^ ERROR `drone_core::reg::Urt: std::marker::Copy` is not satisfied
  assert_clone::<foo::Bar<Urt>>();
  //~^ ERROR `drone_core::reg::Urt: std::clone::Clone` is not satisfied
  assert_copy::<foo::Bar<Srt>>();
  //~^ ERROR `drone_core::reg::Srt: std::marker::Copy` is not satisfied
  assert_clone::<foo::Bar<Srt>>();
  //~^ ERROR `drone_core::reg::Srt: std::clone::Clone` is not satisfied
  assert_copy::<foo::Bar<Frt>>();
  //~^ ERROR `drone_core::reg::Frt: std::marker::Copy` is not satisfied
  assert_clone::<foo::Bar<Frt>>();
  //~^ ERROR `drone_core::reg::Frt: std::clone::Clone` is not satisfied
}