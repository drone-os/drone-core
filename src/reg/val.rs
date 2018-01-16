use super::*;
use core::nonzero::Zeroable;

/// Wrapper for a register value.
pub trait RegVal: Sized + Send + Sync + Clone + Copy + 'static {
  /// Raw integer type.
  type Raw: RegRaw;

  /// A reset value for the register.
  const DEFAULT: Self::Raw;

  /// Creates a new `RegVal` from the raw value.
  unsafe fn from_raw(raw: Self::Raw) -> Self;

  /// Returns the inner integer.
  fn raw(&self) -> Self::Raw;

  /// Returns a mutable reference to the inner integer.
  fn raw_mut(&mut self) -> &mut Self::Raw;

  /// Creates a new `RegVal` from the reset value.
  #[inline(always)]
  unsafe fn default() -> Self {
    Self::from_raw(Self::DEFAULT)
  }

  /// Reads the state of the bit at `offset`.
  ///
  /// # Safety
  ///
  /// `offset` must be less than the size of [`Raw`] in bits.
  ///
  /// [`Raw`]: #associatedtype.Raw
  #[inline(always)]
  unsafe fn read_bit(&self, offset: Self::Raw) -> bool {
    !(self.raw() & bit_at(offset)).is_zero()
  }

  /// Sets the bit at `offset`.
  ///
  /// # Safety
  ///
  /// `offset` must be less than the size of [`Raw`] in bits.
  ///
  /// [`Raw`]: #associatedtype.Raw
  #[inline(always)]
  unsafe fn set_bit(&mut self, offset: Self::Raw) {
    *self.raw_mut() = self.raw() | bit_at(offset);
  }

  /// Clears the bit at `offset`.
  ///
  /// # Safety
  ///
  /// `offset` must be less than the size of [`Raw`] in bits.
  ///
  /// [`Raw`]: #associatedtype.Raw
  #[inline(always)]
  unsafe fn clear_bit(&mut self, offset: Self::Raw) {
    *self.raw_mut() = self.raw() & !bit_at(offset);
  }

  /// Toggles the bit at `offset`.
  ///
  /// # Safety
  ///
  /// `offset` must be less than the size of [`Raw`] in bits.
  ///
  /// [`Raw`]: #associatedtype.Raw
  #[inline(always)]
  unsafe fn toggle_bit(&mut self, offset: Self::Raw) {
    *self.raw_mut() = self.raw() ^ bit_at(offset);
  }

  /// Reads `width` number of low order bits at the `offset` position.
  ///
  /// # Safety
  ///
  /// * `offset` must be less than the size of [`Raw`] in bits.
  /// * `width + offset` must be less than or equal to the size of [`Raw`] in
  /// bits.
  ///
  /// [`Raw`]: #associatedtype.Raw
  #[inline(always)]
  unsafe fn read_bits(&self, offset: Self::Raw, width: Self::Raw) -> Self::Raw {
    if width == Self::Raw::size() {
      self.raw()
    } else {
      self.raw() >> offset & bit_mask(width)
    }
  }

  /// Copies `width` number of low order bits from `bits` into the same number
  /// of adjacent bits at `offset` position.
  ///
  /// # Safety
  ///
  /// * `offset` must be less than the size of [`Raw`] in bits.
  /// * `width + offset` must be less than or equal to the size of [`Raw`] in
  /// bits.
  ///
  /// [`Raw`]: #associatedtype.Raw
  #[inline(always)]
  unsafe fn write_bits(
    &mut self,
    offset: Self::Raw,
    width: Self::Raw,
    bits: Self::Raw,
  ) {
    *self.raw_mut() = if width == Self::Raw::size() {
      bits
    } else {
      self.raw() & !(bit_mask(width) << offset)
        | (bits & bit_mask(width)) << offset
    };
  }
}

#[inline(always)]
fn bit_at<T: RegRaw>(offset: T) -> T {
  T::one() << offset
}

#[inline(always)]
fn bit_mask<T: RegRaw>(width: T) -> T {
  bit_at(width) - T::one()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[derive(Clone, Copy)]
  struct Val(u8);

  impl RegVal for Val {
    type Raw = u8;

    const DEFAULT: u8 = 0xAA;

    unsafe fn from_raw(raw: u8) -> Self {
      Val(raw)
    }

    fn raw(&self) -> u8 {
      self.0
    }

    fn raw_mut(&mut self) -> &mut u8 {
      &mut self.0
    }
  }

  #[test]
  fn default() {
    assert_eq!(unsafe { Val::default().raw() }, 0xAA);
  }

  #[test]
  fn read_bit() {
    let val = unsafe { Val::from_raw(0b1010_1010) };
    assert!(!unsafe { val.read_bit(0) });
    assert!(unsafe { val.read_bit(1) });
    assert!(!unsafe { val.read_bit(2) });
    assert!(unsafe { val.read_bit(3) });
    assert!(!unsafe { val.read_bit(4) });
    assert!(unsafe { val.read_bit(5) });
    assert!(!unsafe { val.read_bit(6) });
    assert!(unsafe { val.read_bit(7) });
  }

  #[test]
  fn set_bit() {
    let mut val = unsafe { Val::from_raw(0b1010_1010) };
    unsafe {
      val.set_bit(0);
      val.set_bit(7);
      val.set_bit(4);
      val.set_bit(3);
    }
    assert_eq!(val.raw(), 0b1011_1011);
  }

  #[test]
  fn clear_bit() {
    let mut val = unsafe { Val::from_raw(0b1010_1010) };
    unsafe {
      val.clear_bit(0);
      val.clear_bit(7);
      val.clear_bit(4);
      val.clear_bit(3);
    }
    assert_eq!(val.raw(), 0b0010_0010);
  }

  #[test]
  fn toggle_bit() {
    let mut val = unsafe { Val::from_raw(0b1010_1010) };
    unsafe {
      val.toggle_bit(0);
      val.toggle_bit(7);
      val.toggle_bit(4);
      val.toggle_bit(3);
    }
    assert_eq!(val.raw(), 0b0011_0011);
  }

  #[test]
  fn read_bits() {
    let val = unsafe { Val::from_raw(0b1010_0110) };
    assert_eq!(unsafe { val.read_bits(0, 0) }, 0b0);
    assert_eq!(unsafe { val.read_bits(1, 0) }, 0b0);
    assert_eq!(unsafe { val.read_bits(2, 0) }, 0b0);
    assert_eq!(unsafe { val.read_bits(0, 1) }, 0b0);
    assert_eq!(unsafe { val.read_bits(1, 1) }, 0b1);
    assert_eq!(unsafe { val.read_bits(2, 1) }, 0b1);
    assert_eq!(unsafe { val.read_bits(3, 1) }, 0b0);
    assert_eq!(unsafe { val.read_bits(4, 1) }, 0b0);
    assert_eq!(unsafe { val.read_bits(5, 1) }, 0b1);
    assert_eq!(unsafe { val.read_bits(6, 1) }, 0b0);
    assert_eq!(unsafe { val.read_bits(7, 1) }, 0b1);
    assert_eq!(unsafe { val.read_bits(0, 4) }, 0b0110);
    assert_eq!(unsafe { val.read_bits(1, 4) }, 0b0011);
    assert_eq!(unsafe { val.read_bits(2, 4) }, 0b1001);
    assert_eq!(unsafe { val.read_bits(3, 4) }, 0b0100);
    assert_eq!(unsafe { val.read_bits(4, 4) }, 0b1010);
    assert_eq!(unsafe { val.read_bits(0, 7) }, 0b0100_110);
    assert_eq!(unsafe { val.read_bits(1, 7) }, 0b1010_011);
    assert_eq!(unsafe { val.read_bits(0, 8) }, 0b1010_0110);
  }

  #[test]
  fn write_bits() {
    let mut val = unsafe { Val::from_raw(0b1010_0110) };
    unsafe { val.write_bits(0, 0, 0b0) };
    unsafe { val.write_bits(1, 0, 0b1) };
    unsafe { val.write_bits(6, 0, 0b11) };
    unsafe { val.write_bits(7, 0, 0b111) };
    assert_eq!(val.raw(), 0b1010_0110);
    let mut val = unsafe { Val::from_raw(0b1010_0110) };
    unsafe { val.write_bits(0, 1, 0b1) };
    unsafe { val.write_bits(1, 1, 0b0) };
    unsafe { val.write_bits(7, 1, 0b0) };
    unsafe { val.write_bits(5, 1, 0b1) };
    assert_eq!(val.raw(), 0b0010_0101);
    let mut val = unsafe { Val::from_raw(0b1010_0110) };
    unsafe { val.write_bits(0, 4, 0b1001) };
    assert_eq!(val.raw(), 0b1010_1001);
    unsafe { val.write_bits(1, 4, 0b1001) };
    assert_eq!(val.raw(), 0b1011_0011);
    unsafe { val.write_bits(2, 4, 0b1001) };
    assert_eq!(val.raw(), 0b1010_0111);
    unsafe { val.write_bits(3, 4, 0b1001) };
    assert_eq!(val.raw(), 0b1100_1111);
    unsafe { val.write_bits(4, 4, 0b1001) };
    assert_eq!(val.raw(), 0b1001_1111);
    let mut val = unsafe { Val::from_raw(0b1010_0110) };
    unsafe { val.write_bits(0, 7, 0b1010_011) };
    assert_eq!(val.raw(), 0b1101_0011);
    unsafe { val.write_bits(1, 7, 0b1010_011) };
    assert_eq!(val.raw(), 0b1010_0111);
    let mut val = unsafe { Val::from_raw(0b0001_1000) };
    unsafe { val.write_bits(0, 8, 0b1111_1111) };
    assert_eq!(val.raw(), 0b1111_1111);
  }
}
