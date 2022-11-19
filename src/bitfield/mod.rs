//! A [`Bitfield`] is an integer value treated as a sequence of bits, which can
//! be toggled individually.
//!
//! A type with named bit-fields can be defined with the attribute macro:
//!
//! ```
//! use drone_core::bitfield::Bitfield;
//!
//! #[derive(Clone, Copy, Bitfield)]
//! #[bitfield(
//!     // The syntax of the field definitions is the following:
//!     //     field_name(mode, offset[, width[, doc_string]])
//!     // `width` is default to 1 when omitted.
//!     // `mode` is one of `r` (for read-only), `rw` (for read-write),
//!     //                  `w` (for write-only).
//!     foo(rw, 1, 4, "4-bits field"),
//!     bar(rw, 5, 1, "1-bit field"),
//! )]
//! // The choice of the underlying integer determines the total number of bits.
//! // Available sizes: `u8`, `u16`, `u32`, `u64`, `u128`.
//! struct MyValue(u8);
//!
//! //                          * foo bit
//! let mut value = MyValue(0b0011_1010);
//! //                           * *** bar bits
//!
//! // The size of the value is exactly the size of the underlying integer.
//! assert_eq!(core::mem::size_of_val(&value), 1);
//!
//! // For one-bit fields, the macro defines the following methods:
//! //     value.bar() for reading the bit (except `w` mode)
//! //     value.set_bar() for setting the bit (except `r` mode)
//! //     value.clear_bar() for clearing the bit (except `r` mode)
//! //     value.toggle_bar() for toggling the bit (except `r` mode)
//! assert!(value.bar());
//! value.clear_bar();
//! assert!(!value.bar());
//! value.set_bar();
//! assert!(value.bar());
//! value.toggle_bar();
//! assert!(!value.bar());
//!
//! // For multiple-bit fields, the macro defines the following methods:
//! //     value.foo() for reading the bits (except `w` mode)
//! //     value.write_foo(bits) for writing the bits (except `r` mode)
//! assert_eq!(value.foo(), 0b1101);
//! value.write_foo(0b1010);
//! assert_eq!(value.foo(), 0b1010);
//!
//! assert_eq!(value.0, 0b0001_0100);
//! ```

mod bits;

pub use self::bits::Bits;
/// Defines a new [`Bitfield`].
///
/// See [the module level documentation](self) for details.
#[doc(inline)]
pub use drone_core_macros::Bitfield;

/// An integer value treated as a sequence of bits, which can be toggled
/// individually.
///
/// See [the module level documentation](self) for more.
pub trait Bitfield: Sized + Send + Sync + Clone + Copy + 'static {
    /// The type of the integer. Determines the total number of bits.
    type Bits: Bits;

    /// Returns a copy of the underlying integer.
    fn bits(&self) -> Self::Bits;

    /// Returns a mutable reference to the underlying integer.
    fn bits_mut(&mut self) -> &mut Self::Bits;

    /// Returns `true` if the bit at `offset` is set.
    ///
    /// # Safety
    ///
    /// `offset` must not exceed the integer size.
    #[inline]
    unsafe fn read_bit(&self, offset: Self::Bits) -> bool {
        !(self.bits() & bit_at(offset)).is_zero()
    }

    /// Sets the bit at `offset`.
    ///
    /// # Safety
    ///
    /// `offset` must not exceed the integer size.
    #[inline]
    unsafe fn set_bit(&mut self, offset: Self::Bits) {
        *self.bits_mut() = self.bits() | bit_at(offset);
    }

    /// Clears the bit at `offset`.
    ///
    /// # Safety
    ///
    /// `offset` must not exceed the integer size.
    #[inline]
    unsafe fn clear_bit(&mut self, offset: Self::Bits) {
        *self.bits_mut() = self.bits() & !bit_at(offset);
    }

    /// Toggles the bit at `offset`.
    ///
    /// # Safety
    ///
    /// `offset` must not exceed the integer size.
    #[inline]
    unsafe fn toggle_bit(&mut self, offset: Self::Bits) {
        *self.bits_mut() = self.bits() ^ bit_at(offset);
    }

    /// Writes the bit at `offset`
    ///
    /// # Safety
    ///
    /// `offset` must not exceed the integer size.
    #[inline]
    unsafe fn write_bit(&mut self, offset: Self::Bits, bit: bool) {
        *self.bits_mut() = self.bits() & !bit_at(offset) | maybe_bit_at(bit, offset);
    }

    /// Returns `width` number of bits at `offset` position.
    ///
    /// # Safety
    ///
    /// `offset + width` must not exceed the integer size.
    #[inline]
    unsafe fn read_bits(&self, offset: Self::Bits, width: Self::Bits) -> Self::Bits {
        if width == Self::Bits::width() {
            self.bits()
        } else {
            self.bits() >> offset & bit_mask(width)
        }
    }

    /// Writes `width` number of bits at `offset` position from `bits`.
    ///
    /// # Safety
    ///
    /// `offset + width` must not exceed the integer size.
    #[inline]
    unsafe fn write_bits(&mut self, offset: Self::Bits, width: Self::Bits, bits: Self::Bits) {
        *self.bits_mut() = if width == Self::Bits::width() {
            bits
        } else {
            self.bits() & !(bit_mask(width) << offset) | (bits & bit_mask(width)) << offset
        };
    }
}

fn maybe_bit_at<T: Bits>(bit: bool, offset: T) -> T {
    T::from_usize(bit.into()) << offset
}

fn bit_at<T: Bits>(offset: T) -> T {
    maybe_bit_at(true, offset)
}

fn bit_mask<T: Bits>(width: T) -> T {
    bit_at(width) - T::from_usize(1)
}
