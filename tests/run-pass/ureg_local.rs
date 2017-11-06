#![feature(decl_macro)]

extern crate drone;

use drone::reg;
use drone::reg::prelude::*;
use std as core;

reg!(0xDEAD_BEEF 0x20 TestReg RReg WReg);

fn assert_rw_local_reg<T: URegLocal>() {}

fn main() {
  assert_rw_local_reg::<TestReg<Lr>>();
}
