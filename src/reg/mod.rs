//! Safe API for memory-mapped registers.

pub mod flavor;

/// Memory-mapped registers prelude.
pub mod prelude {
  pub use super::{RReg, Reg, RegValue, RwLocalReg, WReg};
  pub use super::flavor::{Atomic, Flavor, Local};
}

use core::mem::size_of;
use core::ptr::{read_volatile, write_volatile};

/// Memory-mapped register handler. Types which implement this trait should be
/// zero-sized. This is a zero-cost abstraction for safely working with
/// memory-mapped registers.
pub trait Reg<T>
where
  Self: Sized,
  T: flavor::Flavor,
{
  /// Type that wraps a raw register value.
  type Value: RegValue;

  /// Memory address of the register.
  const ADDRESS: usize;

  /// Register handler constructor. All the safety of the memory-mapped
  /// registers interface is based on a contract that this method must be called
  /// no more than once for a particular register in the whole program.
  unsafe fn attach() -> Self;
}

/// Register that can read its value.
pub trait RReg<T>
where
  Self: Reg<T>,
  T: flavor::Flavor,
{
  /// Reads and wraps a register value from its memory address.
  fn read(&self) -> Self::Value {
    Self::Value::new(self.read_raw())
  }

  /// Reads a raw register value from its memory address.
  fn read_raw(&self) -> usize {
    unsafe { read_volatile(self.to_ptr()) }
  }

  /// Returns an unsafe constant pointer to the register's memory address.
  fn to_ptr(&self) -> *const usize {
    Self::ADDRESS as *const usize
  }
}

/// Register that can write its value.
pub trait WReg<T>
where
  Self: Reg<T>,
  T: flavor::Flavor,
{
  /// Writes a wrapped register value to its memory address.
  fn write(&mut self, value: Self::Value) {
    self.write_raw(value.raw());
  }

  /// Calls `f` on a blank value and writes the result value to the register's
  /// memory address.
  fn write_with<F>(&mut self, f: F)
  where
    F: FnOnce(&mut Self::Value) -> &Self::Value,
  {
    self.write_raw(f(&mut Self::Value::blank()).raw());
  }

  /// Writes a raw register value to its memory address.
  fn write_raw(&mut self, value: usize) {
    unsafe {
      write_volatile(self.to_mut_ptr(), value);
    }
  }

  /// Returns an unsafe mutable pointer to the register's memory address.
  fn to_mut_ptr(&mut self) -> *mut usize {
    Self::ADDRESS as *mut usize
  }
}

/// Register that can read and write its value in a single-threaded context.
pub trait RwLocalReg
where
  Self: RReg<flavor::Local> + WReg<flavor::Local>,
{
  /// Atomically modifies a register's value.
  fn modify<F>(&mut self, f: F)
  where
    F: FnOnce(&mut Self::Value) -> &Self::Value;
}

/// Wrapper for a corresponding register's value.
pub trait RegValue
where
  Self: Sized,
{
  /// Constructs a wrapper from the raw register `value`.
  fn new(value: usize) -> Self;

  /// Returns the raw integer value.
  fn raw(&self) -> usize;

  /// Returns a mutable reference to the raw integer value.
  fn raw_mut(&mut self) -> &mut usize;

  /// Constructs a blank wrapper for the value of `0`.
  fn blank() -> Self {
    Self::new(0)
  }

  /// Checks the set state of the bit of the register's value by `offset`.
  ///
  /// # Panics
  ///
  /// If `offset` is greater than or equals to the platform's word size in bits.
  fn bit(&self, offset: usize) -> bool {
    assert!(offset < size_of::<usize>() * 8);
    self.raw() & 0b1 << offset != 0
  }

  /// Sets or clears the bit of the register's value by `offset`.
  ///
  /// # Panics
  ///
  /// If `offset` is greater than or equals to the platform's word size in bits.
  fn set_bit(&mut self, offset: usize, value: bool) -> &mut Self {
    assert!(offset < size_of::<usize>() * 8);
    let mask = 0b1 << offset;
    if value {
      *self.raw_mut() |= mask;
    } else {
      *self.raw_mut() &= !mask;
    }
    self
  }

  /// Reads the `width` number of low order bits at the `offset` position from
  /// the raw interger value.
  ///
  /// # Panics
  ///
  /// * If `offset` is greater than or equals to the platform's word size in
  ///   bits.
  /// * If `width + offset` is greater than the platform's word size in bits.
  fn bits(&self, offset: usize, width: usize) -> usize {
    assert!(offset < size_of::<usize>() * 8);
    assert!(width <= size_of::<usize>() * 8 - offset);
    self.raw() >> offset & (0b1 << width) - 1
  }

  /// Copies the `width` number of low order bits from the `value` into the same
  /// number of adjacent bits at the `offset` position at the raw integer value.
  ///
  /// # Panics
  ///
  /// * If `offset` is greater than or equals to the platform's word size in
  ///   bits.
  /// * If `width + offset` is greater than the platform's word size in bits.
  /// * If `value` contains bits outside the width range.
  fn set_bits(
    &mut self,
    offset: usize,
    width: usize,
    value: usize,
  ) -> &mut Self {
    assert!(offset < size_of::<usize>() * 8);
    assert!(width <= size_of::<usize>() * 8 - offset);
    assert_eq!(value & !((0b1 << width) - 1), 0);
    *self.raw_mut() &= !((0b1 << width) - 1 << offset);
    *self.raw_mut() |= value << offset;
    self
  }
}

impl<T> RwLocalReg for T
where
  T: RReg<flavor::Local> + WReg<flavor::Local>,
{
  fn modify<F>(&mut self, f: F)
  where
    F: FnOnce(&mut Self::Value) -> &Self::Value,
  {
    let value = f(&mut self.read()).raw();
    self.write_raw(value);
  }
}

#[macro_export]
macro_rules! reg {
  (
    [$address:expr]
    $(#[$reg_meta:meta])* $reg:ident
    $(#[$value_meta:meta])* $value:ident
    $($trait:ident { $($impl:tt)* })*
  ) => {
    $(#[$reg_meta])*
    pub struct $reg<T: $crate::reg::flavor::Flavor> {
      flavor: ::core::marker::PhantomData<T>,
    }

    $(#[$value_meta])*
    pub struct $value {
      value: usize,
    }

    impl<T: $crate::reg::flavor::Flavor> $crate::reg::Reg<T> for $reg<T> {
      type Value = $value;

      const ADDRESS: usize = $address;

      unsafe fn attach() -> Self {
        let flavor = ::core::marker::PhantomData;
        Self { flavor }
      }
    }

    impl $crate::reg::RegValue for $value {
      fn new(value: usize) -> Self {
        Self { value }
      }

      fn raw(&self) -> usize {
        self.value
      }

      fn raw_mut(&mut self) -> &mut usize {
        &mut self.value
      }
    }

    $(
      impl<T: $crate::reg::flavor::Flavor> $trait<T> for $reg<T> {
        $($impl)*
      }
    )*
  };
}

#[cfg(test)]
mod tests {
  use super::*;

  reg!([0xDEAD_BEEF] TestReg TestRegValue RReg {} WReg {});

  #[test]
  fn size_of_reg() {
    assert_eq!(size_of::<TestReg<flavor::Local>>(), 0);
    assert_eq!(size_of::<TestReg<flavor::Atomic>>(), 0);
  }

  #[test]
  fn size_of_reg_value() {
    assert_eq!(size_of::<TestRegValue>(), size_of::<usize>());
  }

  #[test]
  fn reg_value_bit() {
    assert!(!TestRegValue::blank().bit(17));
    assert!(!TestRegValue::blank().bit(0));
    assert!(!TestRegValue::blank().bit(size_of::<usize>() * 8 - 1));
    assert!(!TestRegValue::new(0b1110_1111).bit(4));
    assert!(TestRegValue::new(0b1000_0000).bit(7));
    assert!(TestRegValue::new(0b1).bit(0));
    assert!(
      TestRegValue::new(0b1 << size_of::<usize>() * 8 - 1)
        .bit(size_of::<usize>() * 8 - 1)
    );
  }

  #[test]
  #[should_panic]
  fn reg_value_bit_invalid_offset() {
    TestRegValue::blank().bit(size_of::<usize>() * 8);
  }

  #[test]
  fn reg_value_set_bit() {
    let mut value = TestRegValue::blank();
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
    let mut value = TestRegValue::blank();
    value.set_bit(size_of::<usize>() * 8 - 1, true);
    assert_eq!(value.raw(), 0b1 << size_of::<usize>() * 8 - 1);
    value.set_bit(size_of::<usize>() * 8 - 1, false);
    assert_eq!(value.raw(), 0);
  }

  #[test]
  #[should_panic]
  fn reg_value_set_bit_invalid_offset() {
    TestRegValue::blank().set_bit(size_of::<usize>() * 8, true);
  }

  #[test]
  fn reg_value_bits() {
    assert_eq!(TestRegValue::blank().bits(17, 3), 0);
    assert_eq!(TestRegValue::blank().bits(0, 5), 0);
    assert_eq!(TestRegValue::blank().bits(size_of::<usize>() * 8 - 1, 1), 0);
    assert_eq!(TestRegValue::new(0b1110_0111).bits(3, 2), 0);
    assert_eq!(TestRegValue::new(0b1100_0000).bits(6, 2), 0b11);
    assert_eq!(TestRegValue::new(0b101).bits(0, 3), 0b101);
    assert_eq!(
      TestRegValue::new(0b111 << size_of::<usize>() * 8 - 3)
        .bits(size_of::<usize>() * 8 - 3, 3),
      0b111
    );
  }

  #[test]
  #[should_panic]
  fn reg_value_bits_invalid_offset() {
    TestRegValue::blank().bits(size_of::<usize>() * 8, 1);
  }

  #[test]
  #[should_panic]
  fn reg_value_bits_invalid_width() {
    TestRegValue::blank().bits(size_of::<usize>() * 8 - 1, 2);
  }

  #[test]
  fn reg_value_set_bits() {
    let mut value = TestRegValue::blank();
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
    let mut value = TestRegValue::blank();
    value.set_bits(size_of::<usize>() * 8 - 1, 1, 0b1);
    assert_eq!(value.raw(), 0b1 << size_of::<usize>() * 8 - 1);
    value.set_bits(size_of::<usize>() * 8 - 1, 1, 0);
    assert_eq!(value.raw(), 0);
  }

  #[test]
  #[should_panic]
  fn reg_value_set_bits_invalid_offset() {
    TestRegValue::blank().set_bits(size_of::<usize>() * 8, 1, 0);
  }

  #[test]
  #[should_panic]
  fn reg_value_set_bits_invalid_width() {
    TestRegValue::blank().set_bits(size_of::<usize>() * 8 - 1, 2, 0);
  }

  #[test]
  #[should_panic]
  fn reg_value_set_bits_invalid_value() {
    TestRegValue::blank().set_bits(0, 1, 0b10);
  }
}
