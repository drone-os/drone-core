#![feature(decl_macro)]

extern crate drone;

use drone::reg::mappings;
use drone::reg::prelude::*;

mappings!(FOO BAR { 0xDEAD_BEEF 0x20 0xBEEF_CACE BAZ { 0 1 } });

fn assert_copy<T: Copy>() {}
fn assert_clone<T: Clone>() {}

fn main() {
  assert_copy::<foo::Bar<Cbt>>();
  assert_clone::<foo::Bar<Cbt>>();
}
