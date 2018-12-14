#[macro_use]
extern crate drone_core;

use drone_core::reg;
use drone_core::reg::prelude::*;

reg! {
  pub mod TST TST_RW_REG;
  0xDEAD_BEEF 0x20 0xBEEF_CACE RReg WReg;
  TST_BIT { 0 1 RRRegField WWRegField }
}

reg! {
  pub mod TST TST_RO_REG;
  0xDEAD_BEEF 0x20 0xBEEF_CACE RReg RoReg;
  TST_BIT { 0 1 RRRegField RoRRegField }
}

reg! {
  pub mod TST TST_WO_REG;
  0xDEAD_BEEF 0x20 0xBEEF_CACE WReg WoReg;
  TST_BIT { 0 1 WWRegField WoWRegField }
}

fn assert_rw_reg_unsync<'a, T: RwRegUnsync<'a>>() {}

fn main() {
  assert_rw_reg_unsync::<tst_tst_rw_reg::Reg<Srt>>();
  //~^ ERROR drone_core::reg::WReg<drone_core::reg::Urt>` is not satisfied
  //~| ERROR drone_core::reg::RReg<drone_core::reg::Urt>` is not satisfied
  //~| ERROR drone_core::reg::RegRef<'_, drone_core::reg::Urt>` is not satisfied
  assert_rw_reg_unsync::<tst_tst_ro_reg::Reg<Urt>>();
  //~^ ERROR drone_core::reg::WReg<drone_core::reg::Urt>` is not satisfied
  assert_rw_reg_unsync::<tst_tst_wo_reg::Reg<Urt>>();
  //~^ ERROR drone_core::reg::RReg<drone_core::reg::Urt>` is not satisfied
}
