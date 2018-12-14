#[macro_use]
extern crate drone_core;

use drone_core::reg;
use drone_core::reg::prelude::*;

reg! {
  pub mod FOO BAR;
  0xDEAD_BEEF 0x20 0xBEEF_CACE RReg WReg;
  BAZ { 0 1 RRRegField WWRegField }
}

fn assert_rw_reg_unsync<'a, T: RwRegUnsync<'a>>() {}

fn main() {
  assert_rw_reg_unsync::<foo_bar::Reg<Urt>>();
}
