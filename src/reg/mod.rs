//! Safe API for memory-mapped registers.

pub mod prelude;
pub mod flavor;

use core::fmt::Debug;
use core::mem::size_of;
use core::ops::{BitAnd, BitAndAssign, BitOrAssign, Not, Shl, Shr, Sub};
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
  fn read_raw(&self) -> <Self::Value as RegValue>::Raw {
    unsafe { read_volatile(self.to_ptr()) }
  }

  /// Returns an unsafe constant pointer to the register's memory address.
  fn to_ptr(&self) -> *const <Self::Value as RegValue>::Raw {
    Self::ADDRESS as *const <Self::Value as RegValue>::Raw
  }
}

/// Register that can write its value.
pub trait WReg<T>
where
  Self: Reg<T>,
  T: flavor::Flavor,
{
  /// Writes a wrapped register value to its memory address.
  fn write(&self, value: &Self::Value) {
    self.write_raw(value.raw());
  }

  /// Calls `f` on a default value and writes the result to the register's
  /// memory address.
  fn write_with<F>(&self, f: F)
  where
    F: FnOnce(&mut Self::Value) -> &Self::Value,
  {
    self.write(f(&mut Self::Value::default()));
  }

  /// Writes a raw register value to its memory address.
  fn write_raw(&self, value: <Self::Value as RegValue>::Raw) {
    unsafe {
      write_volatile(self.to_mut_ptr(), value);
    }
  }

  /// Returns an unsafe mutable pointer to the register's memory address.
  fn to_mut_ptr(&self) -> *mut <Self::Value as RegValue>::Raw {
    Self::ADDRESS as *mut <Self::Value as RegValue>::Raw
  }
}

/// Register that can read and write its value in a single-threaded context.
pub trait RwLocalReg
where
  Self: RReg<flavor::Local> + WReg<flavor::Local>,
{
  /// Atomically modifies a register's value.
  fn modify<F>(&self, f: F)
  where
    F: FnOnce(&mut Self::Value) -> &Self::Value;
}

/// Wrapper for a corresponding register's value.
pub trait RegValue
where
  Self: Sized + Default,
{
  /// Raw integer type.
  type Raw: RegRaw;

  /// Constructs a wrapper from the raw register `value`.
  fn new(value: Self::Raw) -> Self;

  /// Returns the raw integer value.
  fn raw(&self) -> Self::Raw;

  /// Returns a mutable reference to the raw integer value.
  fn raw_mut(&mut self) -> &mut Self::Raw;

  /// Checks the set state of the bit of the register's value by `offset`.
  ///
  /// # Panics
  ///
  /// If `offset` is greater than or equals to the platform's word size in bits.
  fn bit(&self, offset: Self::Raw) -> bool {
    assert!(offset < Self::Raw::size_in_bits());
    self.raw() & Self::Raw::one() << offset != Self::Raw::zero()
  }

  /// Sets or clears the bit of the register's value by `offset`.
  ///
  /// # Panics
  ///
  /// If `offset` is greater than or equals to the platform's word size in bits.
  fn set_bit(&mut self, offset: Self::Raw, value: bool) -> &mut Self {
    assert!(offset < Self::Raw::size_in_bits());
    let mask = Self::Raw::one() << offset;
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
  fn bits(&self, offset: Self::Raw, width: Self::Raw) -> Self::Raw {
    assert!(offset < Self::Raw::size_in_bits());
    assert!(width <= Self::Raw::size_in_bits() - offset);
    self.raw() >> offset & (Self::Raw::one() << width) - Self::Raw::one()
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
    offset: Self::Raw,
    width: Self::Raw,
    value: Self::Raw,
  ) -> &mut Self {
    assert!(offset < Self::Raw::size_in_bits());
    assert!(width <= Self::Raw::size_in_bits() - offset);
    assert_eq!(
      value & !((Self::Raw::one() << width) - Self::Raw::one()),
      Self::Raw::zero()
    );
    *self.raw_mut() &=
      !((Self::Raw::one() << width) - Self::Raw::one() << offset);
    *self.raw_mut() |= value << offset;
    self
  }
}

/// Raw register value type.
pub trait RegRaw
where
  Self: Debug
    + Copy
    + Default
    + PartialOrd
    + BitAndAssign
    + BitOrAssign
    + BitAnd<Output = Self>
    + Not<Output = Self>
    + Sub<Output = Self>
    + Shl<Self, Output = Self>
    + Shr<Self, Output = Self>,
{
  /// Size of the type in bits.
  fn size_in_bits() -> Self;

  /// Returns zero.
  fn zero() -> Self;

  /// Returns one.
  fn one() -> Self;
}

impl<T> RwLocalReg for T
where
  T: RReg<flavor::Local> + WReg<flavor::Local>,
{
  fn modify<F>(&self, f: F)
  where
    F: FnOnce(&mut Self::Value) -> &Self::Value,
  {
    self.write(f(&mut self.read()));
  }
}

macro_rules! impl_reg_raw {
  ($type:ty) => {
    impl RegRaw for $type {
      fn size_in_bits() -> $type {
        size_of::<$type>() as $type * 8
      }

      fn zero() -> $type {
        0
      }

      fn one() -> $type {
        1
      }
    }
  };
}

impl_reg_raw!(u64);
impl_reg_raw!(u32);
impl_reg_raw!(u16);
impl_reg_raw!(u8);

#[macro_export]
macro_rules! reg {
  (
    [$address:expr] $raw:ident
    $(#[$reg_meta:meta])* $reg:ident
    $(#[$value_meta:meta])* $value:ident
    $($trait:ident { $($impl:tt)* })*
  ) => {
    $(#[$reg_meta])*
    pub struct $reg<T: $crate::reg::flavor::Flavor> {
      flavor: ::core::marker::PhantomData<T>,
    }

    $(#[$value_meta])*
    #[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
    pub struct $value {
      value: $raw,
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
      type Raw = $raw;

      fn new(value: $raw) -> Self {
        Self { value }
      }

      fn raw(&self) -> $raw {
        self.value
      }

      fn raw_mut(&mut self) -> &mut $raw {
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

  reg!([0xDEAD_BEEF] u32 TestReg TestRegValue RReg {} WReg {});

  #[test]
  fn size_of_reg() {
    assert_eq!(size_of::<TestReg<flavor::Local>>(), 0);
    assert_eq!(size_of::<TestReg<flavor::Atomic>>(), 0);
  }

  #[test]
  fn size_of_reg_value() {
    assert_eq!(size_of::<TestRegValue>(), 4);
  }

  #[test]
  fn reg_value_bit() {
    assert!(!TestRegValue::default().bit(17));
    assert!(!TestRegValue::default().bit(0));
    assert!(!TestRegValue::default().bit(31));
    assert!(!TestRegValue::new(0b1110_1111).bit(4));
    assert!(TestRegValue::new(0b1000_0000).bit(7));
    assert!(TestRegValue::new(0b1).bit(0));
    assert!(TestRegValue::new(0b1 << 31).bit(31));
  }

  #[test]
  #[should_panic]
  fn reg_value_bit_invalid_offset() {
    TestRegValue::default().bit(32);
  }

  #[test]
  fn reg_value_set_bit() {
    let mut value = TestRegValue::default();
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
    let mut value = TestRegValue::default();
    value.set_bit(31, true);
    assert_eq!(value.raw(), 0b1 << 31);
    value.set_bit(31, false);
    assert_eq!(value.raw(), 0);
  }

  #[test]
  #[should_panic]
  fn reg_value_set_bit_invalid_offset() {
    TestRegValue::default().set_bit(32, true);
  }

  #[test]
  fn reg_value_bits() {
    assert_eq!(TestRegValue::default().bits(17, 3), 0);
    assert_eq!(TestRegValue::default().bits(0, 5), 0);
    assert_eq!(TestRegValue::default().bits(31, 1), 0);
    assert_eq!(TestRegValue::new(0b1110_0111).bits(3, 2), 0);
    assert_eq!(TestRegValue::new(0b1100_0000).bits(6, 2), 0b11);
    assert_eq!(TestRegValue::new(0b101).bits(0, 3), 0b101);
    assert_eq!(TestRegValue::new(0b111 << 29).bits(29, 3), 0b111);
  }

  #[test]
  #[should_panic]
  fn reg_value_bits_invalid_offset() {
    TestRegValue::default().bits(32, 1);
  }

  #[test]
  #[should_panic]
  fn reg_value_bits_invalid_width() {
    TestRegValue::default().bits(31, 2);
  }

  #[test]
  fn reg_value_set_bits() {
    let mut value = TestRegValue::default();
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
    let mut value = TestRegValue::default();
    value.set_bits(31, 1, 0b1);
    assert_eq!(value.raw(), 0b1 << 31);
    value.set_bits(31, 1, 0);
    assert_eq!(value.raw(), 0);
  }

  #[test]
  #[should_panic]
  fn reg_value_set_bits_invalid_offset() {
    TestRegValue::default().set_bits(32, 1, 0);
  }

  #[test]
  #[should_panic]
  fn reg_value_set_bits_invalid_width() {
    TestRegValue::default().set_bits(31, 2, 0);
  }

  #[test]
  #[should_panic]
  fn reg_value_set_bits_invalid_value() {
    TestRegValue::default().set_bits(0, 1, 0b10);
  }
}
