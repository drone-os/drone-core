#![feature(decl_macro)]

extern crate drone;

use drone::reg::mappings;
use drone::reg::prelude::*;

mappings!(FOO BAR { 0xDEAD_BEEF 0x20 0xBEEF_CACE BAZ { 0 1 } });

fn assert_send<T: Send>() {}
fn assert_sync<T: Sync>() {}

fn main() {
  assert_send::<foo::Bar<Ubt>>();
  assert_sync::<foo::Bar<Ubt>>();
  //~^ ERROR `drone::reg::Ubt: drone::prelude::Sync` is not satisfied
}
