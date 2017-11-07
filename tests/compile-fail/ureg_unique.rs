#![feature(decl_macro)]

extern crate drone;

use drone::reg;
use drone::reg::prelude::*;
use std as core;

reg!(0xDEAD_BEEF 0x20 RwReg RReg WReg);
reg!(0xDEAD_BEEF 0x20 RoReg RReg);
reg!(0xDEAD_BEEF 0x20 WoReg WReg);

fn assert_ureg_unique<T: URegUnique>() {}

fn main() {
  assert_ureg_unique::<RwReg<Sr>>();
  //~^ ERROR drone::reg::WReg<drone::reg::Ur>` is not satisfied
  //~| ERROR drone::reg::RReg<drone::reg::Ur>` is not satisfied
  assert_ureg_unique::<RoReg<Ur>>();
  //~^ ERROR drone::reg::WReg<drone::reg::Ur>` is not satisfied
  assert_ureg_unique::<WoReg<Ur>>();
  //~^ ERROR drone::reg::RReg<drone::reg::Ur>` is not satisfied
}
