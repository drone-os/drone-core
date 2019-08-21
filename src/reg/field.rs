//! Memory-mapped register fields module.
//!
//! See [the top-level module documentation](self) for details.

use crate::{
    bitfield::{Bitfield, Bits},
    reg::{
        tag::{Crt, RegAtomic, RegTag, Srt, Urt},
        RReg, Reg, WReg, WoReg,
    },
    token::Token,
};
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
    fn into_unsync(self) -> Self
    where
        Self: RegField<Urt>,
    {
        self
    }

    /// Converts into synchronized register field token.
    #[inline]
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
        unsafe { &*(self as *const Self as *const Self::SRegField) }
    }
}

/// Single-bit register field.
pub trait RegFieldBit<T: RegTag>: RegField<T> {}

/// Multiple-bits register field.
pub trait RegFieldBits<T: RegTag>: RegField<T> {}

/// Readable field of readable register.
pub trait RRRegField<T: RegTag>
where
    Self: RegField<T>,
    Self::Reg: RReg<T>,
{
    /// Reads the value from the register memory to the opaque value type.
    #[inline]
    fn load_val(&self) -> <Self::Reg as Reg<T>>::Val {
        unsafe {
            <Self::Reg as Reg<T>>::Val::from_bits(read_volatile(
                Self::Reg::ADDRESS as *const <<Self::Reg as Reg<T>>::Val as Bitfield>::Bits,
            ))
        }
    }
}

/// Writable field of writable register.
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
}

/// Readable multiple-bits field of readable register.
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

/// Writable multiple-bits field of writable register.
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

/// Write-only multiple-bits field of write-only register.
pub trait WoWoRegFieldBits<T: RegTag>
where
    Self: RegFieldBits<T> + WoWRegField<T>,
    Self::Reg: WoReg<T>,
{
    /// Writes the reset value with the field bits replaced by `bits` into the
    /// register memory.
    fn write_bits(&self, bits: <<Self::Reg as Reg<T>>::Val as Bitfield>::Bits);
}

impl<T, U> WoWoRegField<T> for U
where
    T: RegTag,
    U: WoWRegField<T>,
    U::Reg: WoReg<T>,
{
    #[inline]
    fn default_val(&self) -> <Self::Reg as Reg<T>>::Val {
        unsafe { <Self::Reg as Reg<T>>::Val::default() }
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

impl<T, U> RRRegFieldBit<T> for U
where
    T: RegTag,
    U: RegFieldBit<T> + RRRegField<T>,
    U::Reg: RReg<T>,
{
    #[inline]
    fn read(&self, val: &<Self::Reg as Reg<T>>::Val) -> bool {
        unsafe {
            val.read_bit(<<Self::Reg as Reg<T>>::Val as Bitfield>::Bits::from_usize(
                Self::OFFSET,
            ))
        }
    }

    #[inline]
    fn read_bit(&self) -> bool {
        self.read(&self.load_val())
    }
}

impl<T, U> WWRegFieldBit<T> for U
where
    T: RegTag,
    U: RegFieldBit<T> + WWRegField<T>,
    U::Reg: WReg<T>,
{
    #[inline]
    fn set(&self, val: &mut <Self::Reg as Reg<T>>::Val) {
        unsafe {
            val.set_bit(<<Self::Reg as Reg<T>>::Val as Bitfield>::Bits::from_usize(
                Self::OFFSET,
            ));
        }
    }

    #[inline]
    fn clear(&self, val: &mut <Self::Reg as Reg<T>>::Val) {
        unsafe {
            val.clear_bit(<<Self::Reg as Reg<T>>::Val as Bitfield>::Bits::from_usize(
                Self::OFFSET,
            ));
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
}

impl<T, U> WoWoRegFieldBit<T> for U
where
    T: RegTag,
    U: RegFieldBit<T> + WoWRegField<T>,
    U::Reg: WoReg<T>,
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
}

impl<T, U> RRRegFieldBits<T> for U
where
    T: RegTag,
    U: RegFieldBits<T> + RRRegField<T>,
    U::Reg: RReg<T>,
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

impl<T, U> WWRegFieldBits<T> for U
where
    T: RegTag,
    U: RegFieldBits<T> + WWRegField<T>,
    U::Reg: WReg<T>,
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

impl<T, U> WoWoRegFieldBits<T> for U
where
    T: RegTag,
    U: RegFieldBits<T> + WoWRegField<T>,
    U::Reg: WoReg<T>,
{
    #[inline]
    fn write_bits(&self, bits: <<Self::Reg as Reg<T>>::Val as Bitfield>::Bits) {
        self.store(|val| {
            self.write(val, bits);
        });
    }
}
