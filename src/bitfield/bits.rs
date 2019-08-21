use core::{
    fmt::Debug,
    mem::size_of,
    ops::{BitAnd, BitOr, BitXor, Not, Shl, Shr, Sub},
};

/// An integer interface for [`Bitfield`](super::Bitfield).
///
/// See [the module level documentation](super) for details.
pub trait Bits
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
    /// Creates a new value with the bits of `bits`.
    fn from_usize(bits: usize) -> Self;

    /// Returns the width of the integer type in bits.
    fn width() -> Self;

    /// Returns `true` if all bits of the value are cleared.
    fn is_zero(self) -> bool;
}

macro_rules! bits {
    ($type:ty) => {
        impl Bits for $type {
            #[inline]
            fn from_usize(bits: usize) -> Self {
                bits as Self
            }

            #[inline]
            fn width() -> Self {
                size_of::<Self>() as Self * 8
            }

            #[inline]
            fn is_zero(self) -> bool {
                self == 0
            }
        }
    };
}

bits!(u8);
bits!(u16);
bits!(u32);
bits!(u64);
bits!(u128);
