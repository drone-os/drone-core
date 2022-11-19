//! Memory-mapped register fields module.
//!
//! See [the top-level module documentation](self) for details.

use crate::bitfield::{Bitfield, Bits};
#[cfg(feature = "atomics")]
pub use crate::reg::atomic::{WRwRegFieldAtomic, WRwRegFieldBitAtomic, WRwRegFieldBitsAtomic};
#[cfg(not(feature = "atomics"))]
pub use crate::reg::soft_atomic::{
    WRwRegFieldBitSoftAtomic, WRwRegFieldBitsSoftAtomic, WRwRegFieldSoftAtomic,
};
use crate::reg::tag::{Crt, RegAtomic, RegTag, Srt, Urt};
use crate::reg::{RReg, Reg, WReg, WoReg};
use crate::token::Token;
use core::ptr::{read_volatile, write_volatile};

/// The base trait for a field token of a memory-mapped register.
pub trait RegField<T: RegTag>: Token + Sync {
    /// Parent register token.
    type Reg: Reg<T>;

    /// Corresponding unsynchronized register field token.
    type URegField: RegField<Urt>;

    /// Corresponding synchronized register field token.
    type SRegField: RegField<Srt>;

    /// Corresponding copyable register field token.
    type CRegField: RegField<Crt>;

    /// The offset of the field inside the parent register.
    const OFFSET: usize;

    /// The bit-width of the field.
    const WIDTH: usize;

    /// Converts into unsynchronized register field token.
    #[inline]
    #[must_use]
    fn into_unsync(self) -> Self
    where
        Self: RegField<Urt>,
    {
        self
    }

    /// Converts into synchronized register field token.
    #[inline]
    #[must_use]
    fn into_sync(self) -> Self
    where
        Self: RegField<Srt>,
    {
        self
    }

    /// Converts into copyable register field token.
    #[inline]
    fn into_copy(self) -> Self::CRegField
    where
        T: RegAtomic,
    {
        unsafe { Self::CRegField::take() }
    }

    /// Returns a reference to the synchronized field token.
    #[inline]
    fn as_sync(&self) -> &Self::SRegField
    where
        T: RegAtomic,
    {
        unsafe { &*(self as *const Self).cast::<Self::SRegField>() }
    }
}

/// Single-bit register field.
pub trait RegFieldBit<T: RegTag>: RegField<T> {}

/// Multiple-bits register field.
pub trait RegFieldBits<T: RegTag>: RegField<T> {}

/// Readable field of readable register.
#[allow(clippy::upper_case_acronyms)]
pub trait RRRegField<T: RegTag>
where
    Self: RegField<T>,
    Self::Reg: RReg<T>,
{
    /// Reads the value from the register memory to the opaque value type.
    #[inline]
    fn load_val(&self) -> <Self::Reg as Reg<T>>::Val {
        unsafe {
            Self::Reg::val_from(read_volatile(
                Self::Reg::ADDRESS as *const <<Self::Reg as Reg<T>>::Val as Bitfield>::Bits,
            ))
        }
    }
}

/// Writable field of writable register.
#[allow(clippy::upper_case_acronyms)]
pub trait WWRegField<T: RegTag>
where
    Self: RegField<T>,
    Self::Reg: WReg<T>,
{
}

/// Read-only field of readable register.
pub trait RoRRegField<T: RegTag>
where
    Self: RRRegField<T>,
    Self::Reg: RReg<T>,
{
}

/// Write-only field of writable register.
pub trait WoWRegField<T: RegTag>
where
    Self: WWRegField<T>,
    Self::Reg: WReg<T>,
{
}

/// Write-only field of write-only register.
pub trait WoWoRegField<T: RegTag>
where
    Self: WoWRegField<T>,
    Self::Reg: WoReg<T>,
{
    /// Creates a new opaque register value, and initializes it with the reset
    /// value.
    fn default_val(&self) -> <Self::Reg as Reg<T>>::Val;

    /// Writes an opaque value `val` into the register memory.
    ///
    /// See also [`store`](WoWoRegField::store).
    fn store_val(&self, val: <Self::Reg as Reg<T>>::Val);

    /// Passes the opaque reset value to the closure `f`, then writes the result
    /// of the closure into the register memory.
    ///
    /// See also [`store_val`](WoWoRegField::store_val).
    fn store<F>(&self, f: F)
    where
        F: Fn(&mut <Self::Reg as Reg<T>>::Val);
}

/// Readable single-bit field of readable register.
#[allow(clippy::upper_case_acronyms)]
pub trait RRRegFieldBit<T: RegTag>
where
    Self: RegFieldBit<T> + RRRegField<T>,
    Self::Reg: RReg<T>,
{
    /// Returns `true` if the bit is set in `val`.
    fn read(&self, val: &<Self::Reg as Reg<T>>::Val) -> bool;

    /// Reads the value from the register memory and returns `true` if the bit
    /// is set.
    fn read_bit(&self) -> bool;
}

/// Writable single-bit field of writable register.
#[allow(clippy::upper_case_acronyms)]
pub trait WWRegFieldBit<T: RegTag>
where
    Self: RegFieldBit<T> + WWRegField<T>,
    Self::Reg: WReg<T>,
{
    /// Sets the bit in `val`.
    fn set(&self, val: &mut <Self::Reg as Reg<T>>::Val);

    /// Clears the bit in `val`.
    fn clear(&self, val: &mut <Self::Reg as Reg<T>>::Val);

    /// Toggles the bit in `val`.
    fn toggle(&self, val: &mut <Self::Reg as Reg<T>>::Val);

    /// Writes the bit value in `val`.
    fn write(&self, val: &mut <Self::Reg as Reg<T>>::Val, bit: bool);
}

/// Write-only single-bit field of write-only register.
pub trait WoWoRegFieldBit<T: RegTag>
where
    Self: RegFieldBit<T> + WoWRegField<T>,
    Self::Reg: WoReg<T>,
{
    /// Writes the reset value with the bit set into the register memory.
    fn set_bit(&self);

    /// Writes the reset value with the bit cleared into the register memory.
    fn clear_bit(&self);

    /// Writes the reset value with the bit toggled into the register memory.
    fn toggle_bit(&self);

    /// Writes the reset value with the bit set to `bit` into the register
    /// memory.
    fn write_bit(&self, bit: bool);
}

/// Readable multiple-bit field of readable register.
#[allow(clippy::upper_case_acronyms)]
pub trait RRRegFieldBits<T: RegTag>
where
    Self: RegFieldBits<T> + RRRegField<T>,
    Self::Reg: RReg<T>,
{
    /// Extracts the field bits from `val`.
    fn read(
        &self,
        val: &<Self::Reg as Reg<T>>::Val,
    ) -> <<Self::Reg as Reg<T>>::Val as Bitfield>::Bits;

    /// Reads the value from the register memory and extracts the field bits.
    fn read_bits(&self) -> <<Self::Reg as Reg<T>>::Val as Bitfield>::Bits;
}

/// Writable multiple-bit field of writable register.
#[allow(clippy::upper_case_acronyms)]
pub trait WWRegFieldBits<T: RegTag>
where
    Self: RegFieldBits<T> + WWRegField<T>,
    Self::Reg: WReg<T>,
{
    /// Replaces the field bits in `val` by `bits`.
    fn write(
        &self,
        val: &mut <Self::Reg as Reg<T>>::Val,
        bits: <<Self::Reg as Reg<T>>::Val as Bitfield>::Bits,
    );
}

/// Write-only multiple-bit field of write-only register.
pub trait WoWoRegFieldBits<T: RegTag>
where
    Self: RegFieldBits<T> + WoWRegField<T>,
    Self::Reg: WoReg<T>,
{
    /// Writes the reset value with the field bits replaced by `bits` into the
    /// register memory.
    fn write_bits(&self, bits: <<Self::Reg as Reg<T>>::Val as Bitfield>::Bits);
}

impl<T, R> WoWoRegField<T> for R
where
    T: RegTag,
    R: WoWRegField<T>,
    R::Reg: WoReg<T>,
{
    #[inline]
    fn default_val(&self) -> <Self::Reg as Reg<T>>::Val {
        unsafe { Self::Reg::val_from(<Self::Reg as Reg<T>>::RESET) }
    }

    #[inline]
    fn store_val(&self, val: <Self::Reg as Reg<T>>::Val) {
        unsafe {
            write_volatile(
                Self::Reg::ADDRESS as *mut <<Self::Reg as Reg<T>>::Val as Bitfield>::Bits,
                val.bits(),
            );
        }
    }

    #[inline]
    fn store<F>(&self, f: F)
    where
        F: Fn(&mut <Self::Reg as Reg<T>>::Val),
    {
        let mut val = self.default_val();
        f(&mut val);
        self.store_val(val);
    }
}

impl<T, R> RRRegFieldBit<T> for R
where
    T: RegTag,
    R: RegFieldBit<T> + RRRegField<T>,
    R::Reg: RReg<T>,
{
    #[inline]
    fn read(&self, val: &<Self::Reg as Reg<T>>::Val) -> bool {
        unsafe {
            val.read_bit(<<Self::Reg as Reg<T>>::Val as Bitfield>::Bits::from_usize(Self::OFFSET))
        }
    }

    #[inline]
    fn read_bit(&self) -> bool {
        self.read(&self.load_val())
    }
}

impl<T, R> WWRegFieldBit<T> for R
where
    T: RegTag,
    R: RegFieldBit<T> + WWRegField<T>,
    R::Reg: WReg<T>,
{
    #[inline]
    fn set(&self, val: &mut <Self::Reg as Reg<T>>::Val) {
        unsafe {
            val.set_bit(<<Self::Reg as Reg<T>>::Val as Bitfield>::Bits::from_usize(Self::OFFSET));
        }
    }

    #[inline]
    fn clear(&self, val: &mut <Self::Reg as Reg<T>>::Val) {
        unsafe {
            val.clear_bit(<<Self::Reg as Reg<T>>::Val as Bitfield>::Bits::from_usize(Self::OFFSET));
        }
    }

    #[inline]
    fn toggle(&self, val: &mut <Self::Reg as Reg<T>>::Val) {
        unsafe {
            val.toggle_bit(<<Self::Reg as Reg<T>>::Val as Bitfield>::Bits::from_usize(
                Self::OFFSET,
            ));
        }
    }

    #[inline]
    fn write(&self, val: &mut <Self::Reg as Reg<T>>::Val, bit: bool) {
        unsafe {
            val.write_bit(
                <<Self::Reg as Reg<T>>::Val as Bitfield>::Bits::from_usize(Self::OFFSET),
                bit,
            );
        }
    }
}

impl<T, R> WoWoRegFieldBit<T> for R
where
    T: RegTag,
    R: RegFieldBit<T> + WoWRegField<T>,
    R::Reg: WoReg<T>,
{
    #[inline]
    fn set_bit(&self) {
        self.store(|val| {
            self.set(val);
        });
    }

    #[inline]
    fn clear_bit(&self) {
        self.store(|val| {
            self.clear(val);
        });
    }

    #[inline]
    fn toggle_bit(&self) {
        self.store(|val| {
            self.toggle(val);
        });
    }

    #[inline]
    fn write_bit(&self, bit: bool) {
        self.store(|val| {
            self.write(val, bit);
        });
    }
}

impl<T, R> RRRegFieldBits<T> for R
where
    T: RegTag,
    R: RegFieldBits<T> + RRRegField<T>,
    R::Reg: RReg<T>,
{
    #[inline]
    fn read(
        &self,
        val: &<Self::Reg as Reg<T>>::Val,
    ) -> <<Self::Reg as Reg<T>>::Val as Bitfield>::Bits {
        unsafe {
            val.read_bits(
                <<Self::Reg as Reg<T>>::Val as Bitfield>::Bits::from_usize(Self::OFFSET),
                <<Self::Reg as Reg<T>>::Val as Bitfield>::Bits::from_usize(Self::WIDTH),
            )
        }
    }

    #[inline]
    fn read_bits(&self) -> <<Self::Reg as Reg<T>>::Val as Bitfield>::Bits {
        self.read(&self.load_val())
    }
}

impl<T, R> WWRegFieldBits<T> for R
where
    T: RegTag,
    R: RegFieldBits<T> + WWRegField<T>,
    R::Reg: WReg<T>,
{
    #[inline]
    fn write(
        &self,
        val: &mut <Self::Reg as Reg<T>>::Val,
        bits: <<Self::Reg as Reg<T>>::Val as Bitfield>::Bits,
    ) {
        unsafe {
            val.write_bits(
                <<Self::Reg as Reg<T>>::Val as Bitfield>::Bits::from_usize(Self::OFFSET),
                <<Self::Reg as Reg<T>>::Val as Bitfield>::Bits::from_usize(Self::WIDTH),
                bits,
            );
        }
    }
}

impl<T, R> WoWoRegFieldBits<T> for R
where
    T: RegTag,
    R: RegFieldBits<T> + WoWRegField<T>,
    R::Reg: WoReg<T>,
{
    #[inline]
    fn write_bits(&self, bits: <<Self::Reg as Reg<T>>::Val as Bitfield>::Bits) {
        self.store(|val| {
            self.write(val, bits);
        });
    }
}
