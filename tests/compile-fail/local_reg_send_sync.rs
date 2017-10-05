#![feature(decl_macro)]

extern crate drone;

use drone::reg;
use drone::reg::prelude::*;
use std as core;

reg!(0xDEAD_BEEF 0x20 TestReg);

fn assert_send<T: Send>() {}
fn assert_sync<T: Sync>() {}

fn main() {
  assert_send::<TestReg<Lr>>();
  assert_sync::<TestReg<Lr>>();
  //~^ ERROR `drone::reg::Lr: std::marker::Sync` is not satisfied
}
