use crate::bitfield::Bitfield;
use crate::reg::field::{RegFieldBit, RegFieldBits, WWRegField, WWRegFieldBit, WWRegFieldBits};
use crate::reg::tag::RegAtomic;
use crate::reg::{RReg, Reg, RegHold, WReg, WRegAtomic};
use core::sync::atomic::{AtomicU16, AtomicU32, AtomicU8, Ordering};

/// Atomic operations for read-write register.
pub trait RwRegAtomic<T: RegAtomic>: RReg<T> + WRegAtomic<T> {
    /// Reads the value from the register memory, then passes the value to the
    /// closure `f`, then writes the result of the closure back to the register
    /// memory.
    ///
    /// This operation is atomic, it repeats itself in case it was interrupted
    /// in the middle. Thus the closure `f` may be called multiple times.
    ///
    /// See also [`modify_reg`](RwRegAtomic::modify_reg).
    fn modify<'a, F>(&'a self, f: F)
    where
        F: for<'b> Fn(&'b mut <Self as Reg<T>>::Hold<'a>) -> &'b mut <Self as Reg<T>>::Hold<'a>;

    /// Reads the value from the register memory, then passes a reference to
    /// this register token and the value to the closure `f`, then writes the
    /// modified value into the register memory.
    ///
    /// See also [`modify`](RwRegAtomic::modify).
    fn modify_reg<'a, F>(&'a self, f: F)
    where
        F: for<'b> Fn(&'b Self, &'b mut Self::Val);
}

/// Atomic operations for writable field of read-write register.
pub trait WRwRegFieldAtomic<T: RegAtomic>
where
    Self: WWRegField<T>,
    Self::Reg: RReg<T> + WReg<T>,
{
    /// Reads the value from the register memory, then passes the value to the
    /// closure `f`, then writes the modified value back to the register memory.
    ///
    /// This operation is atomic, it repeats itself in case it was interrupted
    /// in the middle. Thus the closure `f` may be called multiple times.
    fn modify<F>(&self, f: F)
    where
        F: Fn(&mut <Self::Reg as Reg<T>>::Val);
}

/// Atomic operations for writable single-bit field of read-write register.
pub trait WRwRegFieldBitAtomic<T: RegAtomic>
where
    Self: WRwRegFieldAtomic<T> + RegFieldBit<T>,
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

/// Atomic operations for writable multiple-bit field of read-write register.
pub trait WRwRegFieldBitsAtomic<T: RegAtomic>
where
    Self: WRwRegFieldAtomic<T> + RegFieldBits<T>,
    Self::Reg: RReg<T> + WReg<T>,
{
    /// Reads the value from the register memory, replaces the field bits by
    /// `bits`, writes the value back to the register memory, repeat if
    /// interrupted.
    fn write_bits(&self, bits: <<Self::Reg as Reg<T>>::Val as Bitfield>::Bits);
}

pub trait AtomicBits: Sized {
    fn atomic_load(&mut self) -> Self;

    fn atomic_compare_exchange_weak(&mut self, current: Self, new: Self) -> Result<Self, Self>;
}

impl<T, R> RwRegAtomic<T> for R
where
    T: RegAtomic,
    R: RReg<T> + WRegAtomic<T>,
    <R::Val as Bitfield>::Bits: AtomicBits,
{
    #[inline]
    fn modify<'a, F>(&'a self, f: F)
    where
        F: for<'b> Fn(&'b mut <Self as Reg<T>>::Hold<'a>) -> &'b mut <Self as Reg<T>>::Hold<'a>,
    {
        let mut old = unsafe { atomic_load::<T, Self>() };
        loop {
            let mut new = unsafe { self.hold(Self::val_from(old)) };
            f(&mut new);
            match unsafe { atomic_compare_exchange_weak::<T, Self>(old, new.val().bits()) } {
                Ok(_) => break,
                Err(x) => old = x,
            }
        }
    }

    #[inline]
    fn modify_reg<'a, F>(&'a self, f: F)
    where
        F: for<'b> Fn(&'b Self, &'b mut Self::Val),
    {
        let mut old = unsafe { atomic_load::<T, Self>() };
        loop {
            let mut new = unsafe { Self::val_from(old) };
            f(self, &mut new);
            match unsafe { atomic_compare_exchange_weak::<T, Self>(old, new.bits()) } {
                Ok(_) => break,
                Err(x) => old = x,
            }
        }
    }
}

impl<T, R> WRwRegFieldAtomic<T> for R
where
    T: RegAtomic,
    R: WWRegField<T>,
    R::Reg: RReg<T> + WReg<T>,
    <<R::Reg as Reg<T>>::Val as Bitfield>::Bits: AtomicBits,
{
    #[inline]
    fn modify<F>(&self, f: F)
    where
        F: Fn(&mut <Self::Reg as Reg<T>>::Val),
    {
        let mut old = unsafe { atomic_load::<T, Self::Reg>() };
        loop {
            let mut new = unsafe { Self::Reg::val_from(old) };
            f(&mut new);
            match unsafe { atomic_compare_exchange_weak::<T, Self::Reg>(old, new.bits()) } {
                Ok(_) => break,
                Err(x) => old = x,
            }
        }
    }
}

impl<T, R> WRwRegFieldBitAtomic<T> for R
where
    T: RegAtomic,
    R: WRwRegFieldAtomic<T> + RegFieldBit<T>,
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

impl<T, R> WRwRegFieldBitsAtomic<T> for R
where
    T: RegAtomic,
    R: WRwRegFieldAtomic<T> + RegFieldBits<T>,
    R::Reg: RReg<T> + WReg<T>,
{
    #[inline]
    fn write_bits(&self, bits: <<Self::Reg as Reg<T>>::Val as Bitfield>::Bits) {
        self.modify(|val| {
            self.write(val, bits);
        });
    }
}

unsafe fn atomic_load<T, R>() -> <R::Val as Bitfield>::Bits
where
    T: RegAtomic,
    R: Reg<T>,
    <R::Val as Bitfield>::Bits: AtomicBits,
{
    <R::Val as Bitfield>::Bits::atomic_load(unsafe {
        &mut *(R::ADDRESS as *mut <R::Val as Bitfield>::Bits)
    })
}

unsafe fn atomic_compare_exchange_weak<T, R>(
    current: <R::Val as Bitfield>::Bits,
    new: <R::Val as Bitfield>::Bits,
) -> Result<<R::Val as Bitfield>::Bits, <R::Val as Bitfield>::Bits>
where
    T: RegAtomic,
    R: Reg<T>,
    <R::Val as Bitfield>::Bits: AtomicBits,
{
    <R::Val as Bitfield>::Bits::atomic_compare_exchange_weak(
        unsafe { &mut *(R::ADDRESS as *mut <R::Val as Bitfield>::Bits) },
        current,
        new,
    )
}

macro_rules! atomic_bits {
    ($int:ty, $atomic:ty) => {
        impl AtomicBits for $int {
            fn atomic_load(&mut self) -> Self {
                <$atomic>::from_mut(self).load(Ordering::Relaxed)
            }

            fn atomic_compare_exchange_weak(
                &mut self,
                current: Self,
                new: Self,
            ) -> Result<Self, Self> {
                <$atomic>::from_mut(self).compare_exchange_weak(
                    current,
                    new,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                )
            }
        }
    };
}

atomic_bits!(u32, AtomicU32);
atomic_bits!(u16, AtomicU16);
atomic_bits!(u8, AtomicU8);
