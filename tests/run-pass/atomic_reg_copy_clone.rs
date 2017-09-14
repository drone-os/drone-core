#[macro_use]
extern crate drone;

use drone::reg::prelude::*;
use std as core;

reg!([0xDEAD_BEEF] u32 TestReg TestRegValue);

fn assert_copy<T: Copy>() {}
fn assert_clone<T: Clone>() {}

fn main() {
  assert_copy::<TestReg<Ar>>();
  assert_clone::<TestReg<Ar>>();
}
