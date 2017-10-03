#![feature(proc_macro)]

extern crate drone;
extern crate drone_macros;

use drone::reg::prelude::*;
use drone_macros::reg;
use std as core;

reg!(0xDEAD_BEEF 0x20 TestReg);

fn assert_send<T: Send>() {}
fn assert_sync<T: Sync>() {}

fn main() {
  assert_send::<TestReg<Ar>>();
  assert_sync::<TestReg<Ar>>();
}
