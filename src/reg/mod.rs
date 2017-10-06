//! Memory-mapped registers support.
//!
//! # Definition
//!
//! ```
//! # #![feature(decl_macro)]
//! # fn main() {}
//! # use std as core;
//! use drone::reg;
//! use drone::reg::prelude::*;
//!
//! reg! {
//!   //! SysTick control and status register.
//!   0xE000_E010 // memory address
//!   0x20 // bit size
//!   Ctrl // register's name
//!   RReg WReg // list of marker traits to implement
//! }
//! ```
//!
//! [`reg!`]: ../macro.reg.html

pub mod prelude;

mod flavor;

pub use self::flavor::{Ar, Lr, RegFlavor};
pub use drone_macros::reg_imp;

use core::fmt::Debug;
use core::mem::size_of;
use core::ops::{BitAnd, BitAndAssign, BitOrAssign, Not, Shl, Shr, Sub};
use core::ptr::{read_volatile, write_volatile};

/// Memory-mapped register binding. Types which implement this trait should be
/// zero-sized. This is a zero-cost abstraction for safely working with
/// memory-mapped registers.
pub trait Reg<T>
where
  Self: Sized,
  T: RegFlavor,
{
  /// Type that wraps a raw register value.
  type Value: RegValue;

  /// Memory address of the register.
  const ADDRESS: usize;

  /// Register binding constructor. All the safety of the memory-mapped
  /// registers interface is based on a contract that this method must be called
  /// no more than once for a particular register in the whole program.
  unsafe fn bind() -> Self;
}

/// Register that can read its value.
pub trait RReg<T>
where
  Self: Reg<T>,
  T: RegFlavor,
{
  /// Reads and wraps a register value from its memory address.
  #[inline]
  fn read(&self) -> Self::Value {
    Self::Value::new(self.read_raw())
  }

  /// Reads a raw register value from its memory address.
  #[inline]
  fn read_raw(&self) -> <Self::Value as RegValue>::Raw {
    unsafe { read_volatile(self.to_ptr()) }
  }

  /// Returns an unsafe constant pointer to the register's memory address.
  #[inline]
  fn to_ptr(&self) -> *const <Self::Value as RegValue>::Raw {
    Self::ADDRESS as *const <Self::Value as RegValue>::Raw
  }
}

/// Register that can write its value.
pub trait WReg<T>
where
  Self: Reg<T>,
  T: RegFlavor,
{
  /// Writes a wrapped register value to its memory address.
  #[inline]
  fn write_value(&self, value: &Self::Value) {
    self.write_raw(value.raw());
  }

  /// Calls `f` on a default value and writes the result to the register's
  /// memory address.
  #[inline]
  fn write<F>(&self, f: F)
  where
    F: FnOnce(&mut Self::Value) -> &Self::Value,
  {
    self.write_value(f(&mut Self::Value::default()));
  }

  /// Writes a raw register value to its memory address.
  #[inline]
  fn write_raw(&self, value: <Self::Value as RegValue>::Raw) {
    unsafe {
      write_volatile(self.to_mut_ptr(), value);
    }
  }

  /// Returns an unsafe mutable pointer to the register's memory address.
  #[inline]
  fn to_mut_ptr(&self) -> *mut <Self::Value as RegValue>::Raw {
    Self::ADDRESS as *mut <Self::Value as RegValue>::Raw
  }
}

/// Register that can read and write its value in a single-threaded context.
pub trait RwLocalReg
where
  Self: RReg<Lr> + WReg<Lr>,
{
  /// Atomically modifies a register's value.
  #[inline]
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
  #[inline]
  fn bit(&self, offset: Self::Raw) -> bool {
    assert!(offset < Self::Raw::size_in_bits());
    self.raw() & Self::Raw::one() << offset != Self::Raw::zero()
  }

  /// Sets or clears the bit of the register's value by `offset`.
  ///
  /// # Panics
  ///
  /// If `offset` is greater than or equals to the platform's word size in bits.
  #[inline]
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
  #[inline]
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
  #[inline]
  fn set_bits(
    &mut self,
    offset: Self::Raw,
    width: Self::Raw,
    value: Self::Raw,
  ) -> &mut Self {
    assert!(offset < Self::Raw::size_in_bits());
    assert!(width <= Self::Raw::size_in_bits() - offset);
    if width != Self::Raw::size_in_bits() {
      assert_eq!(
        value & !((Self::Raw::one() << width) - Self::Raw::one()),
        Self::Raw::zero()
      );
      *self.raw_mut() &=
        !((Self::Raw::one() << width) - Self::Raw::one() << offset);
      *self.raw_mut() |= value << offset;
    } else {
      *self.raw_mut() = value;
    }
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
  T: RReg<Lr> + WReg<Lr>,
{
  #[inline]
  fn modify<F>(&self, f: F)
  where
    F: FnOnce(&mut Self::Value) -> &Self::Value,
  {
    self.write_value(f(&mut self.read()));
  }
}

macro_rules! impl_reg_raw {
  ($type:ty) => {
    impl RegRaw for $type {
      #[inline]
      fn size_in_bits() -> $type {
        size_of::<$type>() as $type * 8
      }

      #[inline]
      fn zero() -> $type {
        0
      }

      #[inline]
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

/// Define a memory-mapped register.
///
/// See the [`module-level documentation`] for more details.
///
/// [`module-level documentation`]: reg/index.html
pub macro reg($($tokens:tt)*) {
  $crate::reg::reg_imp!($($tokens)*);
}
