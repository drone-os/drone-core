#![feature(proc_macro)]

extern crate drone_core;

use drone_core::reg::mappings;
use drone_core::reg::prelude::*;
use std::mem::size_of;
use test_block::TestReg;
use test_block::test_reg::Val;

mappings! {
  /// Test block doc attribute
  #[doc = "test block attribute"]
  TEST_BLOCK;
  /// Test reg doc attribute
  #[doc = "test reg attribute"]
  TEST_REG {
    0xDEAD_BEEF 0x20 0xBEEF_CACE RReg WReg;
    TEST_BIT { 0 1 RRegField WRegField }
    TEST_BITS { 1 3 RRegField WRegField }
  }
}

#[test]
fn reg_val_default() {
  unsafe {
    assert_eq!(Val::default().raw(), 0xBEEF_CACE);
  }
}

#[test]
fn size_of_reg() {
  assert_eq!(size_of::<TestReg<Utt>>(), 0);
  assert_eq!(size_of::<TestReg<Stt>>(), 0);
  assert_eq!(size_of::<TestReg<Ftt>>(), 0);
  assert_eq!(size_of::<TestReg<Ctt>>(), 0);
}

#[test]
fn size_of_reg_val() {
  assert_eq!(size_of::<Val>(), 4);
}
