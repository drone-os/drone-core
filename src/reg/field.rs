use super::*;
use bitfield::{Bitfield, Bits};
use core::ptr::{read_volatile, write_volatile};

/// Register field token.
pub trait RegField<T: RegTag>: Sized + Send + Sync + 'static {
  /// Parent register type.
  type Reg: Reg<T>;

  /// Corresponding unsynchronized register field token.
  type URegField: RegField<Urt>;

  /// Corresponding synchronized register field token.
  type SRegField: RegField<Srt>;

  /// Corresponding copyable register field token.
  type CRegField: RegField<Crt>;

  /// Address offset of the field.
  const OFFSET: usize;

  /// Bit-width of the field.
  const WIDTH: usize;

  /// Creates a new rigester field token.
  ///
  /// # Safety
  ///
  /// Must be called only inside an implementation of `Reg`.
  unsafe fn new() -> Self;

  /// Converts to an unsynchronized register field token.
  #[inline(always)]
  fn to_unsync(self) -> Self
  where
    Self: RegField<Urt>,
  {
    self
  }

  /// Converts to a synchronized register field token.
  #[inline(always)]
  fn to_sync(self) -> Self
  where
    Self: RegField<Srt>,
  {
    self
  }

  /// Converts to a copyable register field token.
  #[inline(always)]
  fn to_copy(self) -> Self::CRegField
  where
    T: RegAtomic,
  {
    unsafe { Self::CRegField::new() }
  }

  /// Takes a non-copy and returns a copy register field token.
  #[inline(always)]
  fn acquire_copy(self) -> Self::CRegField
  where
    T: RegOwned + RegAtomic,
  {
    unsafe { Self::CRegField::new() }
  }

  /// Converts to a synchronized register field token reference.
  #[inline(always)]
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

/// Register field that can read its value.
pub trait RRRegField<T: RegTag>
where
  Self: RegField<T>,
  Self::Reg: RReg<T>,
{
  /// Reads a register value from its memory address.
  #[inline(always)]
  fn load_val(&self) -> <Self::Reg as Reg<T>>::Val {
    unsafe {
      <Self::Reg as Reg<T>>::Val::from_bits(read_volatile(
        Self::Reg::ADDRESS
          as *const <<Self::Reg as Reg<T>>::Val as Bitfield>::Bits,
      ))
    }
  }
}

/// Register field that can write its value.
pub trait WWRegField<T: RegTag>
where
  Self: RegField<T>,
  Self::Reg: WReg<T>,
{
}

/// Register field that can only read its value.
pub trait RoRRegField<T: RegTag>
where
  Self: RRRegField<T>,
  Self::Reg: RReg<T>,
{
}

/// Register field that can only write its value.
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
  /// Creates a new reset value.
  fn default_val(&self) -> <Self::Reg as Reg<T>>::Val;

  /// Writes the value `val`.
  fn store_val(&self, val: <Self::Reg as Reg<T>>::Val);

  /// Updates a new reset value with `f` and writes the result to the register's
  /// memory address.
  fn store<F>(&self, f: F)
  where
    F: Fn(&mut <Self::Reg as Reg<T>>::Val);
}

/// Single-bit register field that can read its value.
pub trait RRRegFieldBit<T: RegTag>
where
  Self: RegFieldBit<T> + RRRegField<T>,
  Self::Reg: RReg<T>,
{
  /// Reads the state of the bit from `val`.
  fn read(&self, val: &<Self::Reg as Reg<T>>::Val) -> bool;

  /// Reads the state of the bit from memory.
  fn read_bit(&self) -> bool;
}

/// Single-bit register field that can write its value.
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

/// Single-bit write-only field of write-only register.
pub trait WoWoRegFieldBit<T: RegTag>
where
  Self: RegFieldBit<T> + WoWRegField<T>,
  Self::Reg: WoReg<T>,
{
  /// Sets the bit in memory.
  fn set_bit(&self);

  /// Clears the bit in memory.
  fn clear_bit(&self);

  /// Toggles the bit in memory.
  fn toggle_bit(&self);

  /// An alias for [`set_bit`](WoWoRegFieldBit::set_bit).
  #[inline(always)]
  fn store_set(&self) {
    self.set_bit();
  }

  /// An alias for [`clear_bit`](WoWoRegFieldBit::clear_bit).
  #[inline(always)]
  fn store_clear(&self) {
    self.clear_bit();
  }

  /// An alias for [`toggle_bit`](WoWoRegFieldBit::toggle_bit).
  #[inline(always)]
  fn store_toggle(&self) {
    self.toggle_bit();
  }
}

/// Multiple-bits register field that can read its value.
pub trait RRRegFieldBits<T: RegTag>
where
  Self: RegFieldBits<T> + RRRegField<T>,
  Self::Reg: RReg<T>,
{
  /// Reads the bits from `val`.
  fn read(
    &self,
    val: &<Self::Reg as Reg<T>>::Val,
  ) -> <<Self::Reg as Reg<T>>::Val as Bitfield>::Bits;

  /// Reads the bits from memory.
  fn read_bits(&self) -> <<Self::Reg as Reg<T>>::Val as Bitfield>::Bits;
}

/// Multiple-bits register field that can write its value.
pub trait WWRegFieldBits<T: RegTag>
where
  Self: RegFieldBits<T> + WWRegField<T>,
  Self::Reg: WReg<T>,
{
  /// Write `bits` to `val`.
  fn write(
    &self,
    val: &mut <Self::Reg as Reg<T>>::Val,
    bits: <<Self::Reg as Reg<T>>::Val as Bitfield>::Bits,
  );
}

/// Multiple-bits write-only field of write-only register.
pub trait WoWoRegFieldBits<T: RegTag>
where
  Self: RegFieldBits<T> + WoWRegField<T>,
  Self::Reg: WoReg<T>,
{
  /// Sets the bit in memory.
  fn write_bits(&self, bits: <<Self::Reg as Reg<T>>::Val as Bitfield>::Bits);

  /// An alias for [`write_bits`](WoWoRegFieldBits::write_bits).
  #[inline(always)]
  fn store_bits(&self, bits: <<Self::Reg as Reg<T>>::Val as Bitfield>::Bits) {
    self.write_bits(bits);
  }
}

impl<T, U> WoWoRegField<T> for U
where
  T: RegTag,
  U: WoWRegField<T>,
  U::Reg: WoReg<T>,
{
  #[inline(always)]
  fn default_val(&self) -> <U::Reg as Reg<T>>::Val {
    unsafe { <U::Reg as Reg<T>>::Val::default() }
  }

  #[inline(always)]
  fn store_val(&self, val: <U::Reg as Reg<T>>::Val) {
    unsafe {
      write_volatile(
        U::Reg::ADDRESS as *mut <<U::Reg as Reg<T>>::Val as Bitfield>::Bits,
        val.bits(),
      );
    }
  }

  #[inline(always)]
  fn store<F>(&self, f: F)
  where
    F: Fn(&mut <U::Reg as Reg<T>>::Val),
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
  #[inline(always)]
  fn read(&self, val: &<U::Reg as Reg<T>>::Val) -> bool {
    unsafe {
      val.read_bit(<<U::Reg as Reg<T>>::Val as Bitfield>::Bits::from_usize(
        U::OFFSET,
      ))
    }
  }

  #[inline(always)]
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
  #[inline(always)]
  fn set(&self, val: &mut <U::Reg as Reg<T>>::Val) {
    unsafe {
      val.set_bit(<<U::Reg as Reg<T>>::Val as Bitfield>::Bits::from_usize(
        U::OFFSET,
      ));
    }
  }

  #[inline(always)]
  fn clear(&self, val: &mut <U::Reg as Reg<T>>::Val) {
    unsafe {
      val.clear_bit(<<U::Reg as Reg<T>>::Val as Bitfield>::Bits::from_usize(
        U::OFFSET,
      ));
    }
  }

  #[inline(always)]
  fn toggle(&self, val: &mut <U::Reg as Reg<T>>::Val) {
    unsafe {
      val.toggle_bit(<<U::Reg as Reg<T>>::Val as Bitfield>::Bits::from_usize(
        U::OFFSET,
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
  #[inline(always)]
  fn set_bit(&self) {
    self.store(|val| {
      self.set(val);
    });
  }

  #[inline(always)]
  fn clear_bit(&self) {
    self.store(|val| {
      self.clear(val);
    });
  }

  #[inline(always)]
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
  #[inline(always)]
  fn read(
    &self,
    val: &<U::Reg as Reg<T>>::Val,
  ) -> <<U::Reg as Reg<T>>::Val as Bitfield>::Bits {
    unsafe {
      val.read_bits(
        <<U::Reg as Reg<T>>::Val as Bitfield>::Bits::from_usize(U::OFFSET),
        <<U::Reg as Reg<T>>::Val as Bitfield>::Bits::from_usize(U::WIDTH),
      )
    }
  }

  #[inline(always)]
  fn read_bits(&self) -> <<U::Reg as Reg<T>>::Val as Bitfield>::Bits {
    self.read(&self.load_val())
  }
}

impl<T, U> WWRegFieldBits<T> for U
where
  T: RegTag,
  U: RegFieldBits<T> + WWRegField<T>,
  U::Reg: WReg<T>,
{
  #[inline(always)]
  fn write(
    &self,
    val: &mut <U::Reg as Reg<T>>::Val,
    bits: <<U::Reg as Reg<T>>::Val as Bitfield>::Bits,
  ) {
    unsafe {
      val.write_bits(
        <<U::Reg as Reg<T>>::Val as Bitfield>::Bits::from_usize(U::OFFSET),
        <<U::Reg as Reg<T>>::Val as Bitfield>::Bits::from_usize(U::WIDTH),
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
  #[inline(always)]
  fn write_bits(&self, bits: <<U::Reg as Reg<T>>::Val as Bitfield>::Bits) {
    self.store(|val| {
      self.write(val, bits);
    });
  }
}
