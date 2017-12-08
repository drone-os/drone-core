#![feature(decl_macro)]

extern crate drone;

use drone::reg::mappings;
use drone::reg::prelude::*;

mappings!(FOO BAR { 0xDEAD_BEEF 0x20 0xBEEF_CACE BAZ { 0 1 } });

fn assert_copy<T: Copy>() {}
fn assert_clone<T: Clone>() {}

fn main() {
  assert_copy::<foo::Bar<Ubt>>();
  //~^ ERROR `foo::bar::Reg<drone::reg::Ubt>: drone::prelude::Copy`
  assert_clone::<foo::Bar<Ubt>>();
  //~^ ERROR `foo::bar::Reg<drone::reg::Ubt>: drone::prelude::Clone`
  assert_copy::<foo::Bar<Sbt>>();
  //~^ ERROR `foo::bar::Reg<drone::reg::Sbt>: drone::prelude::Copy`
  assert_clone::<foo::Bar<Sbt>>();
  //~^ ERROR `foo::bar::Reg<drone::reg::Sbt>: drone::prelude::Clone`
  assert_copy::<foo::Bar<Fbt>>();
  //~^ ERROR `foo::bar::Reg<drone::reg::Fbt>: drone::prelude::Copy`
  assert_clone::<foo::Bar<Fbt>>();
  //~^ ERROR `foo::bar::Reg<drone::reg::Fbt>: drone::prelude::Clone`
}
