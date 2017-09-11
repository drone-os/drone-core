#[macro_use]
extern crate drone;

use drone::reg::prelude::*;
use std as core;

reg!([0xDEAD_BEEF] u32 RwReg RwRegValue RReg {} WReg {});
reg!([0xDEAD_BEEF] u32 RoReg RoRegValue RReg {});
reg!([0xDEAD_BEEF] u32 WoReg WoRegValue WReg {});

fn assert_rw_local_reg<T: RwLocalReg>() {}

fn main() {
  assert_rw_local_reg::<RwReg<Atomic>>();
  //~^ ERROR drone::reg::WReg<drone::reg::prelude::Local>` is not satisfied
  //~| ERROR drone::reg::RReg<drone::reg::prelude::Local>` is not satisfied
  assert_rw_local_reg::<RoReg<Local>>();
  //~^ ERROR drone::reg::WReg<drone::reg::prelude::Local>` is not satisfied
  assert_rw_local_reg::<WoReg<Local>>();
  //~^ ERROR drone::reg::RReg<drone::reg::prelude::Local>` is not satisfied
}
