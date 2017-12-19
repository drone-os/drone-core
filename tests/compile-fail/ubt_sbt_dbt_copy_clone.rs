#![feature(decl_macro)]

extern crate drone_core;

use drone_core::reg::mappings;
use drone_core::reg::prelude::*;

mappings!(FOO; BAR { 0xDEAD_BEEF 0x20 0xBEEF_CACE BAZ { 0 1 } });

fn assert_copy<T: Copy>() {}
fn assert_clone<T: Clone>() {}

fn main() {
  assert_copy::<foo::Bar<Ubt>>();
  //~^ ERROR `foo::bar::Reg<drone_core::reg::Ubt>: drone_core::prelude::Copy`
  assert_clone::<foo::Bar<Ubt>>();
  //~^ ERROR `foo::bar::Reg<drone_core::reg::Ubt>: drone_core::prelude::Clone`
  assert_copy::<foo::Bar<Sbt>>();
  //~^ ERROR `foo::bar::Reg<drone_core::reg::Sbt>: drone_core::prelude::Copy`
  assert_clone::<foo::Bar<Sbt>>();
  //~^ ERROR `foo::bar::Reg<drone_core::reg::Sbt>: drone_core::prelude::Clone`
  assert_copy::<foo::Bar<Fbt>>();
  //~^ ERROR `foo::bar::Reg<drone_core::reg::Fbt>: drone_core::prelude::Copy`
  assert_clone::<foo::Bar<Fbt>>();
  //~^ ERROR `foo::bar::Reg<drone_core::reg::Fbt>: drone_core::prelude::Clone`
}
