//! A packed struct of bits, which fits within a single integer.
//!
//! Example:
//!
//! ```
//! use drone_core::bitfield::Bitfield;
//!
//! #[derive(Bitfield, Copy, Clone)]
//! #[bitfield(
//!   foo(rw, 0, 1, "Test read-write bit."),
//!   bar(r, 1, 2, "Test read-only bits."),
//!   baz(w, 3, 3, "Test write-only bits.")
//! )]
//! struct Packed(u8);
//!
//! # fn main() {
//! let mut x = unsafe { Packed::from_bits(0b1111_0100) };
//! assert!(!x.foo());
//! x.toggle_foo();
//! assert!(x.foo());
//! x.clear_foo();
//! assert!(!x.foo());
//! x.set_foo();
//! assert!(x.foo());
//! assert_eq!(x.bar(), 0b10);
//! x.write_baz(0b101);
//! # }
//! ```

mod bits;

pub use self::bits::Bits;
pub use drone_core_macros::Bitfield;

/// A packed struct of bits, which fits within a single integer.
///
/// See [the module level documentation](index.html) for more.
pub trait Bitfield: Sized + Send + Sync + Clone + Copy + 'static {
  /// The underlying integer type.
  type Bits: Bits;

  /// The default value.
  const DEFAULT: Self::Bits;

  /// Creates a new `Bitfield` from raw bits.
  unsafe fn from_bits(bits: Self::Bits) -> Self;

  /// Returns the underlying integer.
  fn bits(&self) -> Self::Bits;

  /// Returns a mutable reference to the underlying integer.
  fn bits_mut(&mut self) -> &mut Self::Bits;

  /// Creates a new `Bitfield` from the default value.
  #[inline]
  unsafe fn default() -> Self {
    Self::from_bits(Self::DEFAULT)
  }

  /// Reads the state of the bit at `offset`.
  ///
  /// # Safety
  ///
  /// * `offset` must be less than the size of [`Bits`] in bits.
  ///
  /// [`Bits`]: Bitfield::Bits
  #[inline]
  unsafe fn read_bit(&self, offset: Self::Bits) -> bool {
    !(self.bits() & bit_at(offset)).is_zero()
  }

  /// Sets the bit at `offset`.
  ///
  /// # Safety
  ///
  /// * `offset` must be less than the size of [`Bits`] in bits.
  ///
  /// [`Bits`]: Bitfield::Bits
  #[inline]
  unsafe fn set_bit(&mut self, offset: Self::Bits) {
    *self.bits_mut() = self.bits() | bit_at(offset);
  }

  /// Clears the bit at `offset`.
  ///
  /// # Safety
  ///
  /// * `offset` must be less than the size of [`Bits`] in bits.
  ///
  /// [`Bits`]: Bitfield::Bits
  #[inline]
  unsafe fn clear_bit(&mut self, offset: Self::Bits) {
    *self.bits_mut() = self.bits() & !bit_at(offset);
  }

  /// Toggles the bit at `offset`.
  ///
  /// # Safety
  ///
  /// * `offset` must be less than the size of [`Bits`] in bits.
  ///
  /// [`Bits`]: Bitfield::Bits
  #[inline]
  unsafe fn toggle_bit(&mut self, offset: Self::Bits) {
    *self.bits_mut() = self.bits() ^ bit_at(offset);
  }

  /// Reads `width` number of low order bits at the `offset` position.
  ///
  /// # Safety
  ///
  /// * `offset` must be less than the size of [`Bits`] in bits.
  /// * `width + offset` must be less than or equal to the size of [`Bits`] in
  /// bits.
  ///
  /// [`Bits`]: Bitfield::Bits
  #[inline]
  unsafe fn read_bits(
    &self,
    offset: Self::Bits,
    width: Self::Bits,
  ) -> Self::Bits {
    if width == Self::Bits::width() {
      self.bits()
    } else {
      self.bits() >> offset & bit_mask(width)
    }
  }

  /// Copies `width` number of low order bits from `bits` into the same number
  /// of adjacent bits at `offset` position.
  ///
  /// # Safety
  ///
  /// * `offset` must be less than the size of [`Bits`] in bits.
  /// * `width + offset` must be less than or equal to the size of [`Bits`] in
  /// bits.
  ///
  /// [`Bits`]: Bitfield::Bits
  #[inline]
  unsafe fn write_bits(
    &mut self,
    offset: Self::Bits,
    width: Self::Bits,
    bits: Self::Bits,
  ) {
    *self.bits_mut() = if width == Self::Bits::width() {
      bits
    } else {
      self.bits() & !(bit_mask(width) << offset)
        | (bits & bit_mask(width)) << offset
    };
  }
}

fn bit_at<T: Bits>(offset: T) -> T {
  T::one() << offset
}

fn bit_mask<T: Bits>(width: T) -> T {
  bit_at(width) - T::one()
}
