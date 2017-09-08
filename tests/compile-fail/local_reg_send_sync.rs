#[macro_use]
extern crate drone;

use drone::reg::prelude::*;
use std as core;

reg!([0xDEAD_BEEF] TestReg TestRegValue);

fn assert_send<T: Send>() {}
fn assert_sync<T: Sync>() {}

fn main() {
  assert_send::<TestReg<Local>>();
  assert_sync::<TestReg<Local>>();
  //~^ ERROR `drone::reg::flavor::Local: std::marker::Sync` is not satisfied
}
