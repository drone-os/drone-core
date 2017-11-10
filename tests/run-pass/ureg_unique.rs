#![feature(decl_macro)]

extern crate drone;

use drone::reg;
use drone::reg::prelude::*;
use std as core;

reg!(0xDEAD_BEEF 0x20 0xBEEF_CACE TEST_REG RReg WReg);

fn assert_ureg_unique<'a, T: URegUnique<'a>>() {}

fn main() {
  assert_ureg_unique::<TestReg<Ur>>();
}
