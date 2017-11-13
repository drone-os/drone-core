#![feature(decl_macro)]

extern crate drone;

use drone::{reg, reg_block};
use drone::reg::prelude::*;

use test_block::*;

reg_block! {
  TEST_BLOCK

  reg! {
    TEST_RW_REG 0xDEAD_BEEF 0x20 0xBEEF_CACE RReg WReg
    TEST_BIT { 0 1 RRegField WRegField }
  }

  reg! {
    TEST_RO_REG 0xDEAD_BEEF 0x20 0xBEEF_CACE RReg RoReg
    TEST_BIT { 0 1 RRegField RoRegField }
  }

  reg! {
    TEST_WO_REG 0xDEAD_BEEF 0x20 0xBEEF_CACE WReg WoReg
    TEST_BIT { 0 1 WRegField WoRegField }
  }
}

fn assert_rw_reg_unique<'a, T: RwRegUnique<'a>>() {}

fn main() {
  assert_rw_reg_unique::<TestRwReg<Sr>>();
  //~^ ERROR drone::reg::WReg<'_, drone::reg::Ur>` is not satisfied
  //~| ERROR drone::reg::RReg<'_, drone::reg::Ur>` is not satisfied
  assert_rw_reg_unique::<TestRoReg<Ur>>();
  //~^ ERROR drone::reg::WReg<'_, drone::reg::Ur>` is not satisfied
  assert_rw_reg_unique::<TestWoReg<Ur>>();
  //~^ ERROR drone::reg::RReg<'_, drone::reg::Ur>` is not satisfied
}
