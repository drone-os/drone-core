#![feature(proc_macro)]

#[macro_use]
extern crate drone_core;

use drone_core::reg::mappings;
use drone_core::reg::prelude::*;

mappings! {
  FOO;
  BAR {
    0xDEAD_BEEF 0x20 0xBEEF_CACE RReg WReg;
    BAZ { 0 1 RRRegField WWRegField }
  }
}

fn assert_rw_reg_unsync<'a, T: RwRegUnsync<'a>>() {}

fn main() {
  assert_rw_reg_unsync::<foo::Bar<Urt>>();
}
