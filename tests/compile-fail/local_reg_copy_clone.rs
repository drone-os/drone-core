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
  assert_copy::<TestReg<Lr>>();
  //~^ ERROR `TestReg<drone::reg::Lr>: std::marker::Copy` is not satisfied
  assert_clone::<TestReg<Lr>>();
  //~^ ERROR `TestReg<drone::reg::Lr>: std::clone::Clone` is not satisfied
}
