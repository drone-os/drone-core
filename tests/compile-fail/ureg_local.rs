#![feature(decl_macro)]

extern crate drone;

use drone::reg;
use drone::reg::prelude::*;
use std as core;

reg!(0xDEAD_BEEF 0x20 RwReg RReg WReg);
reg!(0xDEAD_BEEF 0x20 RoReg RReg);
reg!(0xDEAD_BEEF 0x20 WoReg WReg);

fn assert_rw_local_reg<T: URegLocal>() {}

fn main() {
  assert_rw_local_reg::<RwReg<Sr>>();
  //~^ ERROR drone::reg::WReg<drone::reg::Lr>` is not satisfied
  //~| ERROR drone::reg::RReg<drone::reg::Lr>` is not satisfied
  assert_rw_local_reg::<RoReg<Lr>>();
  //~^ ERROR drone::reg::WReg<drone::reg::Lr>` is not satisfied
  assert_rw_local_reg::<WoReg<Lr>>();
  //~^ ERROR drone::reg::RReg<drone::reg::Lr>` is not satisfied
}
