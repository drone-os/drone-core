#![feature(decl_macro)]

extern crate drone;

use drone::reg;
use drone::reg::prelude::*;

reg!(TEST_BLOCK TEST_REG { 0xDEAD_BEEF 0x20 0xBEEF_CACE TEST_BIT { 0 1 } });

fn assert_send<T: Send>() {}
fn assert_sync<T: Sync>() {}

fn main() {
  assert_send::<test_block::TestReg<Ur>>();
  assert_sync::<test_block::TestReg<Ur>>();
  //~^ ERROR `drone::reg::Ur: drone::prelude::Sync` is not satisfied
}
