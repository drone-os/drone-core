#![feature(decl_macro)]

extern crate drone;

use drone::reg;
use drone::reg::prelude::*;

reg! {
  TEST_BLOCK

  TEST_RW_REG {
    0xDEAD_BEEF 0x20 0xBEEF_CACE RReg WReg
    TEST_BIT { 0 1 RRegField WRegField }
  }

  TEST_RO_REG  {
    0xDEAD_BEEF 0x20 0xBEEF_CACE RReg RoReg
    TEST_BIT { 0 1 RRegField RoRegField }
  }

  TEST_WO_REG  {
    0xDEAD_BEEF 0x20 0xBEEF_CACE WReg WoReg
    TEST_BIT { 0 1 WRegField WoRegField }
  }
}

fn assert_rw_reg_unique<'a, T: RwRegUnique<'a>>() {}

fn main() {
  assert_rw_reg_unique::<test_block::TestRwReg<Srt>>();
  //~^ ERROR drone::reg::WReg<drone::reg::Urt>` is not satisfied
  //~| ERROR drone::reg::RReg<drone::reg::Urt>` is not satisfied
  //~| ERROR drone::reg::RegRef<'_, drone::reg::Urt>` is not satisfied
  assert_rw_reg_unique::<test_block::TestRoReg<Urt>>();
  //~^ ERROR drone::reg::WReg<drone::reg::Urt>` is not satisfied
  assert_rw_reg_unique::<test_block::TestWoReg<Urt>>();
  //~^ ERROR drone::reg::RReg<drone::reg::Urt>` is not satisfied
}
