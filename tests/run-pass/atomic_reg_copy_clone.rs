#![feature(proc_macro)]

extern crate drone;
extern crate drone_macros;

use drone::reg::prelude::*;
use drone_macros::reg;
use std as core;

reg!(0xDEAD_BEEF 0x20 TestReg);

fn assert_copy<T: Copy>() {}
fn assert_clone<T: Clone>() {}

fn main() {
  assert_copy::<TestReg<Ar>>();
  assert_clone::<TestReg<Ar>>();
}
