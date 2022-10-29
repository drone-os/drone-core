//! Software-implemented synchronization for memory-mapped registers.

use crate::bitfield::Bitfield;
use crate::platform::Interrupts;
use crate::reg::field::{RegFieldBit, RegFieldBits, WWRegField, WWRegFieldBit, WWRegFieldBits};
use crate::reg::tag::RegAtomic;
use crate::reg::{RReg, Reg, RegHold, WReg, WRegAtomic};
use core::ptr::{read_volatile, write_volatile};

/// Software-implemented atomic operations for read-write register.
pub trait RwRegSoftAtomic<T: RegAtomic>: RReg<T> + WRegAtomic<T> {
    /// Reads the value from the register memory, then passes the value to the
    /// closure `f`, then writes the result of the closure back to the register
    /// memory.
    ///
    /// This operation is atomic, it temporarily disables interrupts.
    ///
    /// See also [`modify_reg`](RwRegSoftAtomic::modify_reg).
    fn modify<'a, F>(&'a self, f: F)
    where
        F: for<'b> FnOnce(&'b mut <Self as Reg<T>>::Hold<'a>) -> &'b mut <Self as Reg<T>>::Hold<'a>;

    /// Reads the value from the register memory, then passes a reference to
    /// this register token and the value to the closure `f`, then writes the
    /// modified value into the register memory.
    ///
    /// See also [`modify`](RwRegSoftAtomic::modify).
    fn modify_reg<'a, F>(&'a self, f: F)
    where
        F: for<'b> FnOnce(&'b Self, &'b mut Self::Val);
}

/// Software-implemented atomic operations for writable field of read-write
/// register.
pub trait WRwRegFieldSoftAtomic<T: RegAtomic>
where
    Self: WWRegField<T>,
    Self::Reg: RReg<T> + WReg<T>,
{
    /// Reads the value from the register memory, then passes the value to the
    /// closure `f`, then writes the modified value back to the register memory.
    ///
    /// This operation is atomic, it temporarily disables interrupts.
    fn modify<F>(&self, f: F)
    where
        F: FnOnce(&mut <Self::Reg as Reg<T>>::Val);
}

/// Software-implemented atomic operations for writable single-bit field of
/// read-write register.
pub trait WRwRegFieldBitSoftAtomic<T: RegAtomic>
where
    Self: WRwRegFieldSoftAtomic<T> + RegFieldBit<T>,
    Self::Reg: RReg<T> + WReg<T>,
{
    /// Reads the value from the register memory, sets the bit, writes the value
    /// back to the register memory, repeat if interrupted.
    fn set_bit(&self);

    /// Reads the value from the register memory, clears the bit, writes the
    /// value back to the register memory, repeat if interrupted.
    fn clear_bit(&self);

    /// Reads the value from the register memory, toggles the bit, writes the
    /// value back to the register memory, repeat if interrupted.
    fn toggle_bit(&self);
}

/// Software-implemented atomic operations for writable multiple-bit field of
/// read-write register.
pub trait WRwRegFieldBitsSoftAtomic<T: RegAtomic>
where
    Self: WRwRegFieldSoftAtomic<T> + RegFieldBits<T>,
    Self::Reg: RReg<T> + WReg<T>,
{
    /// Reads the value from the register memory, replaces the field bits by
    /// `bits`, writes the value back to the register memory, repeat if
    /// interrupted.
    fn write_bits(&self, bits: <<Self::Reg as Reg<T>>::Val as Bitfield>::Bits);
}

impl<T, R> RwRegSoftAtomic<T> for R
where
    T: RegAtomic,
    R: RReg<T> + WRegAtomic<T>,
{
    #[inline]
    fn modify<'a, F>(&'a self, f: F)
    where
        F: for<'b> FnOnce(&'b mut <Self as Reg<T>>::Hold<'a>) -> &'b mut <Self as Reg<T>>::Hold<'a>,
    {
        Interrupts::paused(|| unsafe {
            write_volatile(self.as_mut_ptr(), f(&mut self.load()).val().bits());
        });
    }

    #[inline]
    fn modify_reg<'a, F>(&'a self, f: F)
    where
        F: for<'b> FnOnce(&'b Self, &'b mut Self::Val),
    {
        Interrupts::paused(|| {
            let mut val = self.load_val();
            f(self, &mut val);
            self.store_val(val);
        });
    }
}

impl<T, R> WRwRegFieldSoftAtomic<T> for R
where
    T: RegAtomic,
    R: WWRegField<T>,
    R::Reg: RReg<T> + WReg<T>,
{
    #[inline]
    fn modify<F>(&self, f: F)
    where
        F: FnOnce(&mut <Self::Reg as Reg<T>>::Val),
    {
        Interrupts::paused(|| unsafe {
            let mut val = Self::Reg::val_from(read_volatile(
                Self::Reg::ADDRESS as *const <<Self::Reg as Reg<T>>::Val as Bitfield>::Bits,
            ));
            f(&mut val);
            write_volatile(
                Self::Reg::ADDRESS as *mut <<Self::Reg as Reg<T>>::Val as Bitfield>::Bits,
                val.bits(),
            );
        });
    }
}

impl<T, R> WRwRegFieldBitSoftAtomic<T> for R
where
    T: RegAtomic,
    R: WRwRegFieldSoftAtomic<T> + RegFieldBit<T>,
    R::Reg: RReg<T> + WReg<T>,
{
    #[inline]
    fn set_bit(&self) {
        self.modify(|val| {
            self.set(val);
        });
    }

    #[inline]
    fn clear_bit(&self) {
        self.modify(|val| {
            self.clear(val);
        });
    }

    #[inline]
    fn toggle_bit(&self) {
        self.modify(|val| {
            self.toggle(val);
        });
    }
}

impl<T, R> WRwRegFieldBitsSoftAtomic<T> for R
where
    T: RegAtomic,
    R: WRwRegFieldSoftAtomic<T> + RegFieldBits<T>,
    R::Reg: RReg<T> + WReg<T>,
{
    #[inline]
    fn write_bits(&self, bits: <<Self::Reg as Reg<T>>::Val as Bitfield>::Bits) {
        self.modify(|val| {
            self.write(val, bits);
        });
    }
}
