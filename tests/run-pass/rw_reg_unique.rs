#![feature(decl_macro)]

extern crate drone;

use drone::reg::mappings;
use drone::reg::prelude::*;

mappings! {
  FOO;
  BAR {
    0xDEAD_BEEF 0x20 0xBEEF_CACE RReg WReg
    BAZ { 0 1 RRegField WRegField }
  }
}

fn assert_rw_reg_unique<'a, T: RwRegUnique<'a>>() {}

fn main() {
  assert_rw_reg_unique::<foo::Bar<Ubt>>();
}
