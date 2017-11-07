#![feature(decl_macro)]

extern crate drone;

use drone::reg;
use drone::reg::prelude::*;
use std as core;

reg!(0xDEAD_BEEF 0x20 TestReg RReg WReg);

fn assert_ureg_unique<T: URegUnique>() {}

fn main() {
  assert_ureg_unique::<TestReg<Ur>>();
}
