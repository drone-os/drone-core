#![feature(decl_macro)]

extern crate drone;

use drone::reg;
use drone::reg::prelude::*;

reg!(TEST_BLOCK TEST_REG { 0xDEAD_BEEF 0x20 0xBEEF_CACE TEST_BIT { 0 1 } });

fn assert_copy<T: Copy>() {}
fn assert_clone<T: Clone>() {}

fn main() {
  assert_copy::<test_block::TestReg<Urt>>();
  //~^ ERROR `test_block::test_reg::Reg<drone::reg::Urt>: drone::prelude::Copy`
  assert_clone::<test_block::TestReg<Urt>>();
  //~^ ERROR `test_block::test_reg::Reg<drone::reg::Urt>: drone::prelude::Clone`
  assert_copy::<test_block::TestReg<Srt>>();
  //~^ ERROR `test_block::test_reg::Reg<drone::reg::Srt>: drone::prelude::Copy`
  assert_clone::<test_block::TestReg<Srt>>();
  //~^ ERROR `test_block::test_reg::Reg<drone::reg::Srt>: drone::prelude::Clone`
  assert_copy::<test_block::TestReg<Drt>>();
  //~^ ERROR `test_block::test_reg::Reg<drone::reg::Drt>: drone::prelude::Copy`
  assert_clone::<test_block::TestReg<Drt>>();
  //~^ ERROR `test_block::test_reg::Reg<drone::reg::Drt>: drone::prelude::Clone`
}
