#![feature(proc_macro)]

extern crate drone_core;

use drone_core::reg::mappings;
use drone_core::reg::prelude::*;

mappings!(FOO; BAR { 0xDEAD_BEEF 0x20 0xBEEF_CACE BAZ { 0 1 } });

fn assert_copy<T: Copy>() {}
fn assert_clone<T: Clone>() {}

fn main() {
  assert_copy::<foo::Bar<Utt>>();
  //~^ ERROR `foo::bar::Reg<drone_core::reg::Utt>: std::marker::Copy` is not
  assert_clone::<foo::Bar<Utt>>();
  //~^ ERROR `foo::bar::Reg<drone_core::reg::Utt>: std::clone::Clone` is not
  assert_copy::<foo::Bar<Stt>>();
  //~^ ERROR `foo::bar::Reg<drone_core::reg::Stt>: std::marker::Copy` is not
  assert_clone::<foo::Bar<Stt>>();
  //~^ ERROR `foo::bar::Reg<drone_core::reg::Stt>: std::clone::Clone` is not
  assert_copy::<foo::Bar<Ftt>>();
  //~^ ERROR `foo::bar::Reg<drone_core::reg::Ftt>: std::marker::Copy` is not
  assert_clone::<foo::Bar<Ftt>>();
  //~^ ERROR `foo::bar::Reg<drone_core::reg::Ftt>: std::clone::Clone` is not
}
