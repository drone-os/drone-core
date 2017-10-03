#![feature(proc_macro)]

extern crate drone;
extern crate drone_macros;

use drone::reg::prelude::*;
use drone_macros::reg;
use std as core;

reg!(0xDEAD_BEEF 0x20 TestReg RReg WReg);

fn assert_rw_local_reg<T: RwLocalReg>() {}

fn main() {
  assert_rw_local_reg::<TestReg<Lr>>();
}
