//! Memory-mapped registers.
//!
//! # Mapping
//!
//! Most of registers should be already mapped by platform crates. These crates
//! should map registers with [`reg!`] macro as follows:
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
//! # Binding
//!
//! It is strongly encouraged to bind registers with a single [`bind!`] block at
//! the very beginning of the application entry point.
//!
//! ```
//! # #![feature(decl_macro)]
//! # use std as core;
//! # mod stk {
//! #   use drone::reg;
//! #   use drone::reg::prelude::*;
//! #   reg!(0xE000_E010 0x20 Ctrl RReg WReg);
//! # }
//! # fn main() {
//! use drone::reg;
//! use drone::reg::prelude::*;
//! use core::mem::size_of_val;
//!
//! reg::bind! {
//!   stk_ctrl: stk::Ctrl<Lr>,
//! }
//!
//! // Use the bindings inside the current scope.
//! assert_eq!(size_of_val(&stk_ctrl), 0);
//! # }
//! ```
//!
//! [`bind!`]: ../macro.bind.html
//! [`reg!`]: ../macro.reg.html

pub mod prelude;

#[doc(hidden)] // FIXME https://github.com/rust-lang/rust/issues/45266
mod flavor;

pub use self::flavor::*;
pub use drone_macros::{bind_imp, reg_imp};

use core::fmt::Debug;
use core::mem::size_of;
use core::ops::{BitAnd, BitOr, Not, Shl, Shr, Sub};
use core::ptr::{read_volatile, write_volatile};

/// Define a memory-mapped register.
///
/// See the [`module-level documentation`] for more details.
///
/// [`module-level documentation`]: reg/index.html
pub macro bind($($tokens:tt)*) {
  $crate::reg::bind_imp!($($tokens)*);
}

/// Define a memory-mapped register.
///
/// See the [`module-level documentation`] for more details.
///
/// [`module-level documentation`]: reg/index.html
pub macro reg($($tokens:tt)*) {
  $crate::reg::reg_imp!($($tokens)*);
}

/// Memory-mapped register binding. Types which implement this trait should be
/// zero-sized. This is a zero-cost abstraction for safely working with
/// memory-mapped registers.
pub trait Reg<T>
where
  Self: Sized,
  T: RegFlavor,
{
  /// Type that wraps a raw register value.
  type Value: RegVal;

  /// Memory address of the register.
  const ADDRESS: usize;

  /// Creates a binding.
  ///
  /// # Safety
  ///
  /// Must be called no more than once.
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
    unsafe { self.read_raw().into() }
  }

  /// Reads a raw register value from its memory address.
  #[inline]
  unsafe fn read_raw(&self) -> <Self::Value as RegVal>::Raw {
    read_volatile(self.to_ptr())
  }

  /// Returns an unsafe constant pointer to the register's memory address.
  #[inline]
  fn to_ptr(&self) -> *const <Self::Value as RegVal>::Raw {
    Self::ADDRESS as *const <Self::Value as RegVal>::Raw
  }
}

/// Register that can write its value.
pub trait WReg<T>
where
  Self: Reg<T>,
  T: RegFlavor,
{
  /// Calls `f` on a default value and writes the result to the register's
  /// memory address.
  #[inline]
  fn write<F>(&self, f: F)
  where
    F: FnOnce(Self::Value) -> Self::Value,
  {
    self.write_val(f(Self::Value::default()));
  }

  /// Writes a wrapped register value to its memory address.
  #[inline]
  fn write_val(&self, value: Self::Value) {
    unsafe { self.write_raw(value.into_raw()) };
  }

  /// Writes a raw register value to its memory address.
  #[inline]
  unsafe fn write_raw(&self, value: <Self::Value as RegVal>::Raw) {
    write_volatile(self.to_mut_ptr(), value);
  }

  /// Returns an unsafe mutable pointer to the register's memory address.
  #[inline]
  fn to_mut_ptr(&self) -> *mut <Self::Value as RegVal>::Raw {
    Self::ADDRESS as *mut <Self::Value as RegVal>::Raw
  }
}

/// Register that can read and write its value in a single-threaded context.
pub trait URegLocal
where
  Self: RReg<Lr> + WReg<Lr>,
{
  /// Atomically updates a register's value.
  fn update<F>(&self, f: F)
  where
    F: FnOnce(Self::Value) -> Self::Value;
}

/// Wrapper for a corresponding register's value.
pub trait RegVal
where
  Self: Sized + Default,
  Self: From<<Self as RegVal>::Raw>,
{
  /// Raw integer type.
  type Raw: RegRaw;

  /// Returns the raw integer value.
  fn into_raw(self) -> Self::Raw;

  /// Checks the set state of the bit of the register's value by `offset`.
  ///
  /// # Panics
  ///
  /// If `offset` is greater than or equals to the platform's word size in bits.
  #[inline]
  unsafe fn bit(self, offset: Self::Raw) -> bool {
    assert!(offset < Self::Raw::size());
    self.into_raw() & Self::Raw::one() << offset != Self::Raw::zero()
  }

  /// Sets or clears the bit of the register's value by `offset`.
  ///
  /// # Panics
  ///
  /// If `offset` is greater than or equals to the platform's word size in bits.
  #[inline]
  unsafe fn set_bit(self, offset: Self::Raw, value: bool) -> Self {
    assert!(offset < Self::Raw::size());
    let mask = Self::Raw::one() << offset;
    if value {
      self.into_raw() | mask
    } else {
      self.into_raw() & !mask
    }.into()
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
  unsafe fn bits(self, offset: Self::Raw, width: Self::Raw) -> Self::Raw {
    assert!(offset < Self::Raw::size());
    assert!(width <= Self::Raw::size() - offset);
    self.into_raw() >> offset & (Self::Raw::one() << width) - Self::Raw::one()
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
  unsafe fn set_bits(
    self,
    offset: Self::Raw,
    width: Self::Raw,
    value: Self::Raw,
  ) -> Self {
    assert!(offset < Self::Raw::size());
    assert!(width <= Self::Raw::size() - offset);
    if width != Self::Raw::size() {
      assert_eq!(
        value & !((Self::Raw::one() << width) - Self::Raw::one()),
        Self::Raw::zero()
      );
      self.into_raw()
        & !((Self::Raw::one() << width) - Self::Raw::one() << offset)
        | value << offset
    } else {
      value
    }.into()
  }
}

/// Raw register value type.
pub trait RegRaw
where
  Self: Debug
    + Copy
    + Default
    + PartialOrd
    + Not<Output = Self>
    + Sub<Output = Self>
    + BitOr<Output = Self>
    + BitAnd<Output = Self>
    + Shl<Self, Output = Self>
    + Shr<Self, Output = Self>,
{
  /// Size of the type in bits.
  fn size() -> Self;

  /// Returns zero.
  fn zero() -> Self;

  /// Returns one.
  fn one() -> Self;
}

impl<T> URegLocal for T
where
  T: RReg<Lr> + WReg<Lr>,
{
  #[inline]
  fn update<F>(&self, f: F)
  where
    F: FnOnce(Self::Value) -> Self::Value,
  {
    self.write_val(f(self.read()));
  }
}

macro impl_reg_raw($type:ty) {
  impl RegRaw for $type {
    #[inline]
    fn size() -> $type {
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
}

impl_reg_raw!(u64);
impl_reg_raw!(u32);
impl_reg_raw!(u16);
impl_reg_raw!(u8);
