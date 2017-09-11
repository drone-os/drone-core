#[macro_use]
extern crate drone;

use drone::reg::prelude::*;
use std as core;

reg!([0xDEAD_BEEF] u32 TestReg TestRegValue RReg {} WReg {});

fn assert_rw_local_reg<T: RwLocalReg>() {}

fn main() {
  assert_rw_local_reg::<TestReg<Local>>();
}
