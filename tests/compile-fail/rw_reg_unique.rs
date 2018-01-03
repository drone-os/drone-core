#![feature(proc_macro)]

extern crate drone_core;

use drone_core::reg::mappings;
use drone_core::reg::prelude::*;

mappings! {
  TEST_BLOCK;

  TEST_RW_REG {
    0xDEAD_BEEF 0x20 0xBEEF_CACE RReg WReg;
    TEST_BIT { 0 1 RRegField WRegField }
  }

  TEST_RO_REG  {
    0xDEAD_BEEF 0x20 0xBEEF_CACE RReg RoReg;
    TEST_BIT { 0 1 RRegField RoRegField }
  }

  TEST_WO_REG  {
    0xDEAD_BEEF 0x20 0xBEEF_CACE WReg WoReg;
    TEST_BIT { 0 1 WRegField WoRegField }
  }
}

fn assert_rw_reg_unique<'a, T: RwRegUnique<'a>>() {}

fn main() {
  assert_rw_reg_unique::<test_block::TestRwReg<Stt>>();
  //~^ ERROR drone_core::reg::WReg<drone_core::reg::Utt>` is not satisfied
  //~| ERROR drone_core::reg::RReg<drone_core::reg::Utt>` is not satisfied
  //~| ERROR drone_core::reg::RegRef<'_, drone_core::reg::Utt>` is not satisfied
  assert_rw_reg_unique::<test_block::TestRoReg<Utt>>();
  //~^ ERROR drone_core::reg::WReg<drone_core::reg::Utt>` is not satisfied
  assert_rw_reg_unique::<test_block::TestWoReg<Utt>>();
  //~^ ERROR drone_core::reg::RReg<drone_core::reg::Utt>` is not satisfied
}
