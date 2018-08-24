#![feature(proc_macro_gen)]

#[macro_use]
extern crate drone_core;

use drone_core::reg::map;
use drone_core::reg::prelude::*;

map!(pub mod FOO; BAR { 0xDEAD_BEEF 0x20 0xBEEF_CACE; BAZ { 0 1 } });

fn assert_copy<T: Copy>() {}
fn assert_clone<T: Clone>() {}

fn main() {
  assert_copy::<foo::Bar<Crt>>();
  assert_clone::<foo::Bar<Crt>>();
}
