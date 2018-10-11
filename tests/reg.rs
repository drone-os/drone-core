#[macro_use]
extern crate drone_core;

use drone_core::bitfield::Bitfield;
use drone_core::reg::map;
use drone_core::reg::prelude::*;
use std::mem::size_of;
use test_block::test_reg::Val;
use test_block::TestReg;

map! {
  /// Test block doc attribute
  #[doc = "test block attribute"]
  pub mod TEST_BLOCK;
  /// Test reg doc attribute
  #[doc = "test reg attribute"]
  TEST_REG {
    0xDEAD_BEEF 0x20 0xBEEF_CACE RReg WReg;
    TEST_BIT { 0 1 RRRegField WWRegField }
    TEST_BITS { 1 3 RRRegField WWRegField }
  }
}

#[test]
fn reg_val_default() {
  unsafe {
    assert_eq!(Val::default().bits(), 0xBEEF_CACE);
  }
}

#[test]
fn size_of_reg() {
  assert_eq!(size_of::<TestReg<Urt>>(), 0);
  assert_eq!(size_of::<TestReg<Srt>>(), 0);
  assert_eq!(size_of::<TestReg<Frt>>(), 0);
  assert_eq!(size_of::<TestReg<Crt>>(), 0);
}

#[test]
fn size_of_reg_val() {
  assert_eq!(size_of::<Val>(), 4);
}
