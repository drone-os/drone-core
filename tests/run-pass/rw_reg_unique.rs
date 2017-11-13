#![feature(decl_macro)]

extern crate drone;

use drone::reg;
use drone::reg::prelude::*;

reg! {
  TEST_BLOCK TEST_REG 0xDEAD_BEEF 0x20 0xBEEF_CACE RReg WReg
  TEST_BIT { 0 1 RRegField WRegField }
}

fn assert_rw_reg_unique<'a, T: RwRegUnique<'a>>() {}

fn main() {
  assert_rw_reg_unique::<TestReg<Ur>>();
}
