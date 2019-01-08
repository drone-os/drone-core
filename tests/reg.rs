#![feature(proc_macro_hygiene)]

use crate::test_block::{test_reg::Val, TestReg};
use drone_core::{bitfield::Bitfield, reg::prelude::*};
use std::mem::size_of;

use drone_core::reg;

reg! {
  /// Test reg doc attribute
  #[doc = "test reg attribute"]
  pub mod TEST_BLOCK TEST_REG;

  0xDEAD_BEEF 0x20 0xBEEF_CACE RReg WReg;

  TEST_BIT { 0 1 RRRegField WWRegField }
  TEST_BITS { 1 3 RRRegField WWRegField }
}

reg::unsafe_tokens! {
  /// Test index doc attribute
  #[doc = "test index attribute"]
  pub macro unsafe_reg_tokens;
  super;;

  /// Test block doc attribute
  #[doc = "test block attribute"]
  pub mod TEST_BLOCK {
    TEST_REG;
  }
}

unsafe_reg_tokens! {
  /// Test index doc attribute
  #[doc = "test index attribute"]
  pub struct Regs;
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
  assert_eq!(size_of::<TestReg<Crt>>(), 0);
}

#[test]
fn size_of_reg_val() {
  assert_eq!(size_of::<Val>(), 4);
}
