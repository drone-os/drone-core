use super::*;

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
  fn raw(&self) -> Self::Raw;

  /// Returns a mutable reference to the inner integer.
  fn raw_mut(&mut self) -> &mut Self::Raw;

  /// Reads the state of the bit at `offset`.
  ///
  /// # Safety
  ///
  /// `offset` must be greater than or equals to the platform's word size in
  /// bits.
  #[inline(always)]
  unsafe fn read_bit(&self, offset: Self::Raw) -> bool {
    self.raw() & Self::Raw::one() << offset != Self::Raw::zero()
  }

  /// Sets the bit at `offset`.
  ///
  /// # Safety
  ///
  /// `offset` must be greater than or equals to the platform's word size in
  /// bits.
  #[inline(always)]
  unsafe fn set_bit(&mut self, offset: Self::Raw) {
    *self.raw_mut() = self.raw() | Self::Raw::one() << offset;
  }

  /// Clears the bit at `offset`.
  ///
  /// # Safety
  ///
  /// `offset` must be greater than or equals to the platform's word size in
  /// bits.
  #[inline(always)]
  unsafe fn clear_bit(&mut self, offset: Self::Raw) {
    *self.raw_mut() = self.raw() & !(Self::Raw::one() << offset);
  }

  /// Toggles the bit at `offset`.
  ///
  /// # Safety
  ///
  /// `offset` must be greater than or equals to the platform's word size in
  /// bits.
  #[inline(always)]
  unsafe fn toggle_bit(&mut self, offset: Self::Raw) {
    *self.raw_mut() = self.raw() ^ Self::Raw::one() << offset;
  }

  /// Reads `width` number of low order bits at the `offset` position.
  ///
  /// # Safety
  ///
  /// * `offset` must be greater than or equals to the platform's word size in
  ///   bits.
  /// * `width + offset` must be greater than the platform's word size in bits.
  #[inline(always)]
  unsafe fn read_bits(&self, offset: Self::Raw, width: Self::Raw) -> Self::Raw {
    self.raw() >> offset & (Self::Raw::one() << width) - Self::Raw::one()
  }

  /// Copies `width` number of low order bits from `bits` into the same number
  /// of adjacent bits at `offset` position.
  ///
  /// # Safety
  ///
  /// * `offset` must be greater than or equals to the platform's word size in
  ///   bits.
  /// * `width + offset` must be greater than the platform's word size in bits.
  /// * `bits` must not contain bits outside the width range.
  #[inline(always)]
  unsafe fn write_bits(
    &mut self,
    offset: Self::Raw,
    width: Self::Raw,
    bits: Self::Raw,
  ) {
    *self.raw_mut() = if width != Self::Raw::size() {
      self.raw() & !((Self::Raw::one() << width) - Self::Raw::one() << offset)
        | bits << offset
    } else {
      bits
    };
  }
}
