#![feature(decl_macro)]

extern crate drone;

use drone::reg::mappings;
use drone::reg::prelude::*;
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
    0xDEAD_BEEF 0x20 0xBEEF_CACE RReg WReg
    TEST_BIT { 0 1 RRegField WRegField }
    TEST_BITS { 1 3 RRegField WRegField }
  }
}

#[test]
fn reg_val_default() {
  unsafe {
    assert_eq!(Val::reset().raw(), 0xBEEF_CACE);
  }
}

#[test]
fn size_of_reg() {
  assert_eq!(size_of::<TestReg<Ubt>>(), 0);
  assert_eq!(size_of::<TestReg<Sbt>>(), 0);
  assert_eq!(size_of::<TestReg<Fbt>>(), 0);
  assert_eq!(size_of::<TestReg<Cbt>>(), 0);
}

#[test]
fn size_of_reg_val() {
  assert_eq!(size_of::<Val>(), 4);
}

#[test]
fn reg_val_read_bit() {
  unsafe {
    assert!(!Val::from_raw(0).read_bit(17));
    assert!(!Val::from_raw(0).read_bit(0));
    assert!(!Val::from_raw(0).read_bit(31));
    assert!(!Val::from_raw(0b1110_1111).read_bit(4));
    assert!(Val::from_raw(0b1000_0000).read_bit(7));
    assert!(Val::from_raw(0b1).read_bit(0));
    assert!(Val::from_raw(0b1 << 31).read_bit(31));
  }
}

#[test]
fn reg_val_write_bit() {
  unsafe {
    let mut value = Val::from_raw(0);
    value.clear_bit(0);
    assert_eq!(value.raw(), 0b0000_0000);
    value.set_bit(6);
    assert_eq!(value.raw(), 0b0100_0000);
    value.set_bit(0);
    assert_eq!(value.raw(), 0b0100_0001);
    value.clear_bit(5);
    assert_eq!(value.raw(), 0b0100_0001);
    value.toggle_bit(6);
    assert_eq!(value.raw(), 0b0000_0001);
    let mut value = Val::from_raw(0);
    value.set_bit(31);
    assert_eq!(value.raw(), 0b1 << 31);
    value.clear_bit(31);
    assert_eq!(value.raw(), 0);
  }
}

#[test]
fn reg_val_read_bits() {
  unsafe {
    assert_eq!(Val::from_raw(0).read_bits(17, 3), 0);
    assert_eq!(Val::from_raw(0).read_bits(0, 5), 0);
    assert_eq!(Val::from_raw(0).read_bits(31, 1), 0);
    assert_eq!(Val::from_raw(0b1110_0111).read_bits(3, 2), 0);
    assert_eq!(Val::from_raw(0b1100_0000).read_bits(6, 2), 0b11);
    assert_eq!(Val::from_raw(0b101).read_bits(0, 3), 0b101);
    assert_eq!(Val::from_raw(0b111 << 29).read_bits(29, 3), 0b111);
  }
}

#[test]
fn reg_val_write_bits() {
  unsafe {
    let mut value = Val::from_raw(0);
    value.write_bits(0, 2, 0);
    assert_eq!(value.raw(), 0b0000_0000);
    value.write_bits(5, 2, 0b11);
    assert_eq!(value.raw(), 0b0110_0000);
    value.write_bits(0, 2, 0b01);
    assert_eq!(value.raw(), 0b0110_0001);
    value.write_bits(3, 2, 0);
    assert_eq!(value.raw(), 0b0110_0001);
    value.write_bits(4, 4, 0);
    assert_eq!(value.raw(), 0b0000_0001);
    let mut value = Val::from_raw(0);
    value.write_bits(31, 1, 0b1);
    assert_eq!(value.raw(), 0b1 << 31);
    value.write_bits(31, 1, 0);
    assert_eq!(value.raw(), 0);
    value.write_bits(0, 32, 0xFFFF_FFFF);
    assert_eq!(value.raw(), 0xFFFF_FFFF);
  }
}
