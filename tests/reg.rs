#![feature(decl_macro)]

extern crate drone;

use drone::reg;
use drone::reg::prelude::*;
use std as core;
use std::mem::size_of;

reg! {
  //! Test doc attribute
  #![doc = "test attribute"]
  0xDEAD_BEEF 0x20 TestReg RReg WReg
}

#[test]
fn size_of_reg() {
  assert_eq!(size_of::<TestReg<Lr>>(), 0);
  assert_eq!(size_of::<TestReg<Ar>>(), 0);
}

#[test]
fn size_of_reg_value() {
  assert_eq!(size_of::<TestRegVal>(), 4);
}

#[test]
fn reg_value_bit() {
  assert!(!TestRegVal::default().bit(17));
  assert!(!TestRegVal::default().bit(0));
  assert!(!TestRegVal::default().bit(31));
  assert!(!TestRegVal::new(0b1110_1111).bit(4));
  assert!(TestRegVal::new(0b1000_0000).bit(7));
  assert!(TestRegVal::new(0b1).bit(0));
  assert!(TestRegVal::new(0b1 << 31).bit(31));
}

#[test]
#[should_panic]
fn reg_value_bit_invalid_offset() {
  TestRegVal::default().bit(32);
}

#[test]
fn reg_value_set_bit() {
  let mut value = TestRegVal::default();
  value.set_bit(0, false);
  assert_eq!(value.raw(), 0b0000_0000);
  value.set_bit(6, true);
  assert_eq!(value.raw(), 0b0100_0000);
  value.set_bit(0, true);
  assert_eq!(value.raw(), 0b0100_0001);
  value.set_bit(5, false);
  assert_eq!(value.raw(), 0b0100_0001);
  value.set_bit(6, false);
  assert_eq!(value.raw(), 0b0000_0001);
  let mut value = TestRegVal::default();
  value.set_bit(31, true);
  assert_eq!(value.raw(), 0b1 << 31);
  value.set_bit(31, false);
  assert_eq!(value.raw(), 0);
}

#[test]
#[should_panic]
fn reg_value_set_bit_invalid_offset() {
  TestRegVal::default().set_bit(32, true);
}

#[test]
fn reg_value_bits() {
  assert_eq!(TestRegVal::default().bits(17, 3), 0);
  assert_eq!(TestRegVal::default().bits(0, 5), 0);
  assert_eq!(TestRegVal::default().bits(31, 1), 0);
  assert_eq!(TestRegVal::new(0b1110_0111).bits(3, 2), 0);
  assert_eq!(TestRegVal::new(0b1100_0000).bits(6, 2), 0b11);
  assert_eq!(TestRegVal::new(0b101).bits(0, 3), 0b101);
  assert_eq!(TestRegVal::new(0b111 << 29).bits(29, 3), 0b111);
}

#[test]
#[should_panic]
fn reg_value_bits_invalid_offset() {
  TestRegVal::default().bits(32, 1);
}

#[test]
#[should_panic]
fn reg_value_bits_invalid_width() {
  TestRegVal::default().bits(31, 2);
}

#[test]
fn reg_value_set_bits() {
  let mut value = TestRegVal::default();
  value.set_bits(0, 2, 0);
  assert_eq!(value.raw(), 0b0000_0000);
  value.set_bits(5, 2, 0b11);
  assert_eq!(value.raw(), 0b0110_0000);
  value.set_bits(0, 2, 0b01);
  assert_eq!(value.raw(), 0b0110_0001);
  value.set_bits(3, 2, 0);
  assert_eq!(value.raw(), 0b0110_0001);
  value.set_bits(4, 4, 0);
  assert_eq!(value.raw(), 0b0000_0001);
  let mut value = TestRegVal::default();
  value.set_bits(31, 1, 0b1);
  assert_eq!(value.raw(), 0b1 << 31);
  value.set_bits(31, 1, 0);
  assert_eq!(value.raw(), 0);
  value.set_bits(0, 32, 0xFFFF_FFFF);
  assert_eq!(value.raw(), 0xFFFF_FFFF);
}

#[test]
#[should_panic]
fn reg_value_set_bits_invalid_offset() {
  TestRegVal::default().set_bits(32, 1, 0);
}

#[test]
#[should_panic]
fn reg_value_set_bits_invalid_width() {
  TestRegVal::default().set_bits(31, 2, 0);
}

#[test]
#[should_panic]
fn reg_value_set_bits_invalid_value() {
  TestRegVal::default().set_bits(0, 1, 0b10);
}
