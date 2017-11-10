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
//! use drone::reg;
//! use drone::reg::prelude::*;
//!
//! reg! {
//!   //! SysTick control and status register.
//!   0xE000_E010 // memory address
//!   0x20 // bit size
//!   0x0000_0000 // reset value
//!   CTRL // register's name
//!   RReg WReg // list of marker traits to implement
//!   /// Counter enable.
//!   ENABLE { // field name
//!     0 // offset
//!     1 // width
//!   }
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
//! #   reg!(0xE000_E010 0x20 0x0000_0000 CTRL RReg WReg);
//! # }
//! # fn main() {
//! use drone::reg;
//! use drone::reg::prelude::*;
//! use core::mem::size_of_val;
//!
//! reg::bind! {
//!   stk_ctrl: stk::Ctrl<Ur>,
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
pub use drone_macros::{bind_impl, reg_impl};

use core::fmt::Debug;
use core::mem::size_of;
use core::ops::{BitAnd, BitOr, BitXor, Not, Shl, Shr, Sub};
use core::ptr::{read_volatile, write_volatile};

/// Define a memory-mapped register.
///
/// See the [`module-level documentation`] for more details.
///
/// [`module-level documentation`]: reg/index.html
pub macro bind($($tokens:tt)*) {
  $crate::reg::bind_impl!($($tokens)*);
}

/// Define a memory-mapped register.
///
/// See the [`module-level documentation`] for more details.
///
/// [`module-level documentation`]: reg/index.html
pub macro reg($($tokens:tt)*) {
  $crate::reg::reg_impl!($($tokens)*);
}

/// Disambiguation for `Reg::Hold::Val`
pub type RegHoldVal<'a, T, U> = <<U as Reg<'a, T>>::Hold as RegHold<
  'a,
  T,
  U,
>>::Val;

/// Disambiguation for `Reg::Hold::Val::Raw`
pub type RegHoldValRaw<'a, T, U> = <RegHoldVal<'a, T, U> as RegVal>::Raw;

/// Disambiguation for `RegField::Reg::Hold`
pub type RegFieldRegHold<'a, T, U> = <<U as RegField<'a, T>>::Reg as Reg<
  'a,
  T,
>>::Hold;

/// Disambiguation for `RegField::Reg::Hold::Val`
pub type RegFieldRegHoldVal<'a, T, U> = RegHoldVal<
  'a,
  T,
  <U as RegField<'a, T>>::Reg,
>;

/// Disambiguation for `RegField::Reg::Hold::Val::Raw`
pub type RegFieldRegHoldValRaw<'a, T, U> =
  <RegFieldRegHoldVal<'a, T, U> as RegVal>::Raw;

/// Memory-mapped register binding. Types which implement this trait should be
/// zero-sized. This is a zero-cost abstraction for safely working with
/// memory-mapped registers.
pub trait Reg<'a, T>
where
  Self: Sized + 'a,
  T: RegFlavor + 'a,
{
  /// Type that wraps a raw register value and a register reference.
  type Hold: RegHold<'a, T, Self>;

  /// Memory address of the register.
  const ADDRESS: usize;

  #[doc(hidden)]
  unsafe fn bind() -> Self;

  /// Creates a new `Hold` for `val`.
  unsafe fn hold(&'a self, val: RegHoldVal<'a, T, Self>) -> Self::Hold {
    Self::Hold::hold(self, val)
  }

  /// Creates a new `Hold` with reset value.
  fn reset_val(&'a self) -> Self::Hold {
    unsafe { self.hold(RegHoldVal::<'a, T, Self>::reset()) }
  }
}

/// Register that can read its value.
pub trait RReg<'a, T>
where
  Self: Reg<'a, T>,
  T: RegFlavor + 'a,
{
  /// Reads and wraps a register value from its memory address.
  #[inline]
  fn read(&'a self) -> Self::Hold {
    unsafe { self.hold(RegHoldVal::<'a, T, Self>::from_raw(self.read_raw())) }
  }

  /// Reads a raw register value from its memory address.
  #[inline]
  unsafe fn read_raw(&self) -> RegHoldValRaw<'a, T, Self> {
    read_volatile(self.to_ptr())
  }

  /// Returns an unsafe constant pointer to the register's memory address.
  #[inline]
  fn to_ptr(&self) -> *const RegHoldValRaw<'a, T, Self> {
    Self::ADDRESS as *const RegHoldValRaw<'a, T, Self>
  }
}

/// Register that can write its value.
pub trait WReg<'a, T>
where
  Self: Reg<'a, T>,
  T: RegFlavor + 'a,
{
  /// Updates a new reset value with `f` and writes the result to the register's
  /// memory address.
  #[inline]
  fn reset<F>(&'a self, f: F)
  where
    F: FnOnce(&mut Self::Hold) -> &mut Self::Hold,
  {
    unsafe {
      self.write(f(&mut self.hold(RegHoldVal::<'a, T, Self>::reset())));
    }
  }

  /// Writes the holded value `val`.
  #[inline]
  fn write(&self, val: &Self::Hold) {
    self.write_val(val.val());
  }

  /// Writes the unbound value `val`.
  #[inline]
  fn write_val(&self, val: RegHoldVal<'a, T, Self>) {
    unsafe { self.write_raw(val.raw()) };
  }

  /// Writes a raw register value to its memory address.
  #[inline]
  unsafe fn write_raw(&self, raw: RegHoldValRaw<'a, T, Self>) {
    write_volatile(self.to_mut_ptr(), raw);
  }

  /// Returns an unsafe mutable pointer to the register's memory address.
  #[inline]
  fn to_mut_ptr(&self) -> *mut RegHoldValRaw<'a, T, Self> {
    Self::ADDRESS as *mut RegHoldValRaw<'a, T, Self>
  }
}

/// Register that can read and write its value in a single-threaded context.
pub trait URegUnique<'a>
where
  Self: RReg<'a, Ur> + WReg<'a, Ur>,
{
  /// Atomically updates a register's value.
  fn update<F>(&'a mut self, f: F)
  where
    F: FnOnce(&mut Self::Hold) -> &mut Self::Hold;
}

/// Register field binding.
pub trait RegField<'a, T>
where
  T: RegFlavor + 'a,
{
  /// Parent register type.
  type Reg: Reg<'a, T>;

  /// Address offset of the field.
  const OFFSET: usize;

  /// Bit-width of the field.
  const WIDTH: usize;

  #[doc(hidden)]
  unsafe fn bind() -> Self;
}

/// Single-bit register field.
pub trait RegFieldBit<'a, T>
where
  Self: RegField<'a, T>,
  T: RegFlavor + 'a,
{
}

/// Multiple-bits register field.
pub trait RegFieldBits<'a, T>
where
  Self: RegField<'a, T>,
  T: RegFlavor + 'a,
{
}

/// Single-bit register field that can read its value.
pub trait RRegFieldBit<'a, T>
where
  Self: RegFieldBit<'a, T>,
  Self::Reg: RReg<'a, T>,
  T: RegFlavor + 'a,
{
  /// Reads the state of the bit.
  fn read(&self, val: RegFieldRegHoldVal<'a, T, Self>) -> bool;
}

/// Single-bit register field that can write its value.
pub trait WRegFieldBit<'a, T>
where
  Self: RegFieldBit<'a, T>,
  Self::Reg: WReg<'a, T>,
  T: RegFlavor + 'a,
{
  /// Sets the bit.
  fn set(
    &self,
    val: RegFieldRegHoldVal<'a, T, Self>,
  ) -> RegFieldRegHoldVal<'a, T, Self>;

  /// Clears the bit.
  fn clear(
    &self,
    val: RegFieldRegHoldVal<'a, T, Self>,
  ) -> RegFieldRegHoldVal<'a, T, Self>;

  /// Toggles the bit.
  fn toggle(
    &self,
    val: RegFieldRegHoldVal<'a, T, Self>,
  ) -> RegFieldRegHoldVal<'a, T, Self>;
}

/// Multiple-bits register field that can read its value.
pub trait RRegFieldBits<'a, T>
where
  Self: RegFieldBits<'a, T>,
  Self::Reg: RReg<'a, T>,
  T: RegFlavor + 'a,
{
  /// Reads the bits.
  fn read(
    &self,
    val: RegFieldRegHoldVal<'a, T, Self>,
  ) -> RegFieldRegHoldValRaw<'a, T, Self>;
}

/// Multiple-bits register field that can write its value.
pub trait WRegFieldBits<'a, T>
where
  Self: RegFieldBits<'a, T>,
  Self::Reg: WReg<'a, T>,
  T: RegFlavor + 'a,
{
  /// Write the bits.
  fn write(
    &self,
    val: RegFieldRegHoldVal<'a, T, Self>,
    bits: RegFieldRegHoldValRaw<'a, T, Self>,
  ) -> RegFieldRegHoldVal<'a, T, Self>;
}

/// Wrapper for a register value that holds register reference.
pub trait RegHold<'a, T, U>
where
  Self: Sized,
  T: RegFlavor + 'a,
  U: Reg<'a, T>,
{
  /// Type that wraps a raw register value.
  type Val: RegVal;

  #[doc(hidden)]
  unsafe fn hold(reg: &'a U, val: Self::Val) -> Self;

  /// Returns the inner value.
  fn val(&self) -> Self::Val;

  /// Replaces the inner value.
  fn set_val(&mut self, val: Self::Val);
}

/// Wrapper for a register value.
pub trait RegVal
where
  Self: Sized,
{
  /// Raw integer type.
  type Raw: RegRaw;

  /// Creates a new `RegVal` from the reset value.
  unsafe fn reset() -> Self;

  /// Creates a new `RegVal` from the raw value.
  unsafe fn from_raw(raw: Self::Raw) -> Self;

  /// Returns the inner integer.
  fn raw(self) -> Self::Raw;

  /// Reads the state of the bit at `offset`.
  ///
  /// # Panics
  ///
  /// If `offset` is greater than or equals to the platform's word size in bits.
  #[inline]
  unsafe fn read_bit(self, offset: Self::Raw) -> bool {
    assert!(offset < Self::Raw::size());
    self.raw() & Self::Raw::one() << offset != Self::Raw::zero()
  }

  /// Sets the bit at `offset`.
  ///
  /// # Panics
  ///
  /// If `offset` is greater than or equals to the platform's word size in bits.
  #[inline]
  unsafe fn set_bit(self, offset: Self::Raw) -> Self {
    assert!(offset < Self::Raw::size());
    Self::from_raw(self.raw() | Self::Raw::one() << offset)
  }

  /// Clears the bit at `offset`.
  ///
  /// # Panics
  ///
  /// If `offset` is greater than or equals to the platform's word size in bits.
  #[inline]
  unsafe fn clear_bit(self, offset: Self::Raw) -> Self {
    assert!(offset < Self::Raw::size());
    Self::from_raw(self.raw() & !(Self::Raw::one() << offset))
  }

  /// Toggles the bit at `offset`.
  ///
  /// # Panics
  ///
  /// If `offset` is greater than or equals to the platform's word size in bits.
  #[inline]
  unsafe fn toggle_bit(self, offset: Self::Raw) -> Self {
    assert!(offset < Self::Raw::size());
    Self::from_raw(self.raw() ^ Self::Raw::one() << offset)
  }

  /// Reads `width` number of low order bits at the `offset` position.
  ///
  /// # Panics
  ///
  /// * If `offset` is greater than or equals to the platform's word size in
  ///   bits.
  /// * If `width + offset` is greater than the platform's word size in bits.
  #[inline]
  unsafe fn read_bits(self, offset: Self::Raw, width: Self::Raw) -> Self::Raw {
    assert!(offset < Self::Raw::size());
    assert!(width <= Self::Raw::size() - offset);
    self.raw() >> offset & (Self::Raw::one() << width) - Self::Raw::one()
  }

  /// Copies `width` number of low order bits from `bits` into the same number
  /// of adjacent bits at `offset` position.
  ///
  /// # Panics
  ///
  /// * If `offset` is greater than or equals to the platform's word size in
  ///   bits.
  /// * If `width + offset` is greater than the platform's word size in bits.
  /// * If `bits` contains bits outside the width range.
  #[inline]
  unsafe fn write_bits(
    self,
    offset: Self::Raw,
    width: Self::Raw,
    bits: Self::Raw,
  ) -> Self {
    assert!(offset < Self::Raw::size());
    assert!(width <= Self::Raw::size() - offset);
    Self::from_raw(if width != Self::Raw::size() {
      assert_eq!(
        bits & !((Self::Raw::one() << width) - Self::Raw::one()),
        Self::Raw::zero()
      );
      self.raw() & !((Self::Raw::one() << width) - Self::Raw::one() << offset)
        | bits << offset
    } else {
      bits
    })
  }
}

/// Raw register value type.
pub trait RegRaw
where
  Self: Sized
    + Debug
    + Copy
    + PartialOrd
    + Not<Output = Self>
    + Sub<Output = Self>
    + BitOr<Output = Self>
    + BitXor<Output = Self>
    + BitAnd<Output = Self>
    + Shl<Self, Output = Self>
    + Shr<Self, Output = Self>,
{
  /// Creates a `RegRaw` from `usize`
  fn from_usize(raw: usize) -> Self;

  /// Size of the type in bits.
  fn size() -> Self;

  /// Returns zero.
  fn zero() -> Self;

  /// Returns one.
  fn one() -> Self;
}

impl<'a, T> URegUnique<'a> for T
where
  T: RReg<'a, Ur> + WReg<'a, Ur>,
{
  #[inline]
  fn update<F>(&'a mut self, f: F)
  where
    F: FnOnce(&mut Self::Hold) -> &mut Self::Hold,
  {
    self.write(f(&mut self.read()));
  }
}

impl<'a, T, U> RRegFieldBit<'a, T> for U
where
  T: RegFlavor + 'a,
  U: RegFieldBit<'a, T>,
  U::Reg: RReg<'a, T>,
{
  #[inline]
  fn read(&self, val: RegFieldRegHoldVal<'a, T, Self>) -> bool {
    unsafe {
      val.read_bit(RegFieldRegHoldValRaw::<'a, T, Self>::from_usize(
        Self::OFFSET,
      ))
    }
  }
}

impl<'a, T, U> WRegFieldBit<'a, T> for U
where
  T: RegFlavor + 'a,
  U: RegFieldBit<'a, T>,
  U::Reg: WReg<'a, T>,
{
  #[inline]
  fn set(
    &self,
    val: RegFieldRegHoldVal<'a, T, Self>,
  ) -> RegFieldRegHoldVal<'a, T, Self> {
    unsafe {
      val.set_bit(RegFieldRegHoldValRaw::<'a, T, Self>::from_usize(
        Self::OFFSET,
      ))
    }
  }

  #[inline]
  fn clear(
    &self,
    val: RegFieldRegHoldVal<'a, T, Self>,
  ) -> RegFieldRegHoldVal<'a, T, Self> {
    unsafe {
      val.clear_bit(RegFieldRegHoldValRaw::<'a, T, Self>::from_usize(
        Self::OFFSET,
      ))
    }
  }

  #[inline]
  fn toggle(
    &self,
    val: RegFieldRegHoldVal<'a, T, Self>,
  ) -> RegFieldRegHoldVal<'a, T, Self> {
    unsafe {
      val.toggle_bit(RegFieldRegHoldValRaw::<'a, T, Self>::from_usize(
        Self::OFFSET,
      ))
    }
  }
}

impl<'a, T, U> RRegFieldBits<'a, T> for U
where
  T: RegFlavor + 'a,
  U: RegFieldBits<'a, T>,
  U::Reg: RReg<'a, T>,
{
  #[inline]
  fn read(
    &self,
    val: RegFieldRegHoldVal<'a, T, Self>,
  ) -> RegFieldRegHoldValRaw<'a, T, Self> {
    unsafe {
      val.read_bits(
        RegFieldRegHoldValRaw::<'a, T, Self>::from_usize(Self::OFFSET),
        RegFieldRegHoldValRaw::<'a, T, Self>::from_usize(Self::WIDTH),
      )
    }
  }
}

impl<'a, T, U> WRegFieldBits<'a, T> for U
where
  T: RegFlavor + 'a,
  U: RegFieldBits<'a, T>,
  U::Reg: WReg<'a, T>,
{
  #[inline]
  fn write(
    &self,
    val: RegFieldRegHoldVal<'a, T, Self>,
    bits: RegFieldRegHoldValRaw<'a, T, Self>,
  ) -> RegFieldRegHoldVal<'a, T, Self> {
    unsafe {
      val.write_bits(
        RegFieldRegHoldValRaw::<'a, T, Self>::from_usize(Self::OFFSET),
        RegFieldRegHoldValRaw::<'a, T, Self>::from_usize(Self::WIDTH),
        bits,
      )
    }
  }
}

macro impl_reg_raw($type:ty) {
  impl RegRaw for $type {
    #[inline]
    fn from_usize(raw: usize) -> Self {
      raw as $type
    }

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
