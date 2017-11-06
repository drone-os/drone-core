#![feature(decl_macro)]

extern crate drone;

use drone::reg;
use drone::reg::prelude::*;
use std as core;

reg!(0xDEAD_BEEF 0x20 TestReg);

fn assert_copy<T: Copy>() {}
fn assert_clone<T: Clone>() {}

fn main() {
  assert_copy::<TestReg<Cr>>();
  assert_clone::<TestReg<Cr>>();
}
