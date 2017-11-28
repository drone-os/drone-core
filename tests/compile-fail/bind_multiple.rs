#![feature(decl_macro)]
#![feature(linkage)]

extern crate drone;

use drone::reg;
use drone::reg::bindings;
use drone::reg::prelude::*;

reg!(TEST_BLOCK TEST_REG { 0xDEAD_BEEF 0x20 0xBEEF_CACE TEST_BIT { 0 1 } });
//~^ ERROR symbol `drone_reg_binding_DEADBEEF` is already defined

bindings! {
  a: test_block::TestReg<Urt>,
  b: test_block::TestReg<Urt>,
}

fn main() {}
