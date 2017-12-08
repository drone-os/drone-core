use core::fmt::Debug;
use core::mem::size_of;
use core::ops::{BitAnd, BitOr, BitXor, Not, Shl, Shr, Sub};

/// Raw register value type.
pub trait RegRaw
  : Sized
  + Debug
  + Copy
  + PartialOrd
  + Not<Output = Self>
  + Sub<Output = Self>
  + BitOr<Output = Self>
  + BitXor<Output = Self>
  + BitAnd<Output = Self>
  + Shl<Self, Output = Self>
  + Shr<Self, Output = Self> {
  /// Creates a `RegRaw` from `usize`
  fn from_usize(raw: usize) -> Self;

  /// Size of the type in bits.
  fn size() -> Self;

  /// Returns zero.
  fn zero() -> Self;

  /// Returns one.
  fn one() -> Self;
}

#[doc(hidden)] // FIXME https://github.com/rust-lang/rust/issues/45266
macro reg_raw($type:ty) {
  impl RegRaw for $type {
    #[inline(always)]
    fn from_usize(raw: usize) -> Self {
      raw as $type
    }

    #[inline(always)]
    fn size() -> $type {
      size_of::<$type>() as $type * 8
    }

    #[inline(always)]
    fn zero() -> $type {
      0
    }

    #[inline(always)]
    fn one() -> $type {
      1
    }
  }
}

reg_raw!(u64);
reg_raw!(u32);
reg_raw!(u16);
reg_raw!(u8);
