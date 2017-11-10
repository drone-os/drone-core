#![feature(decl_macro)]

extern crate drone;

use drone::reg;
use drone::reg::prelude::*;

reg!(TEST_REG 0xDEAD_BEEF 0x20 0xBEEF_CACE);

fn assert_copy<T: Copy>() {}
fn assert_clone<T: Clone>() {}

fn main() {
  assert_copy::<TestReg<Ur>>();
  //~^ ERROR `test_reg::Reg<drone::reg::Ur>: drone::prelude::Copy` is not
  // satisfied
  assert_clone::<TestReg<Ur>>();
  //~^ ERROR `test_reg::Reg<drone::reg::Ur>: drone::prelude::Clone` is not
  // satisfied
  assert_copy::<TestReg<Sr>>();
  //~^ ERROR `test_reg::Reg<drone::reg::Sr>: drone::prelude::Copy` is not
  // satisfied
  assert_clone::<TestReg<Sr>>();
  //~^ ERROR `test_reg::Reg<drone::reg::Sr>: drone::prelude::Clone` is not
  // satisfied
}
