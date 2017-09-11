#[macro_use]
extern crate drone;

use drone::reg::prelude::*;
use std as core;

reg!([0xDEAD_BEEF] u32 TestReg TestRegValue);

fn assert_send<T: Send>() {}
fn assert_sync<T: Sync>() {}

fn main() {
  assert_send::<TestReg<Local>>();
  assert_sync::<TestReg<Local>>();
  //~^ ERROR `drone::reg::prelude::Local: std::marker::Sync` is not satisfied
}
