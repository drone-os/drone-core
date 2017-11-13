use super::*;
use core::ptr::{read_volatile, write_volatile};

/// Disambiguation for `RegField::Reg::Hold`
pub type RegFieldRegHold<'a, T, U> = <<U as RegField<'a, T>>::Reg as Reg<
  'a,
  T,
>>::Hold;

/// Disambiguation for `RegField::Reg::Hold::Val`
pub type RegFieldRegHoldVal<'a, T, U> = RegHoldVal<
  'a,
  T,
  <U as RegField<'a, T>>::Reg,
>;

/// Disambiguation for `RegField::Reg::Hold::Val::Raw`
pub type RegFieldRegHoldValRaw<'a, T, U> =
  <RegFieldRegHoldVal<'a, T, U> as RegVal>::Raw;

/// Set of register fields.
pub trait RegFields<'a, T, U>
where
  Self: Sized,
  T: RegTag + 'a,
  U: Reg<'a, T>,
{
  #[doc(hidden)]
  unsafe fn bind() -> Self;

  /// Converts the set of fields into the register.
  fn into_reg(self) -> U;
}

/// Register field binding.
pub trait RegField<'a, T>
where
  T: RegTag + 'a,
{
  /// Parent register type.
  type Reg: Reg<'a, T>;

  /// Address offset of the field.
  const OFFSET: usize;

  /// Bit-width of the field.
  const WIDTH: usize;

  #[doc(hidden)]
  unsafe fn bind() -> Self;
}

/// Single-bit register field.
pub trait RegFieldBit<'a, T>
where
  Self: RegField<'a, T>,
  T: RegTag + 'a,
{
}

/// Multiple-bits register field.
pub trait RegFieldBits<'a, T>
where
  Self: RegField<'a, T>,
  T: RegTag + 'a,
{
}

/// Register field that can read its value.
pub trait RRegField<'a, T>
where
  Self: RegField<'a, T>,
  Self::Reg: RReg<'a, T>,
  T: RegTag + 'a,
{
  /// Reads a register value from its memory address.
  #[inline]
  fn load_val(&self) -> RegFieldRegHoldVal<'a, T, Self> {
    unsafe {
      RegFieldRegHoldVal::<'a, T, Self>::from_raw(read_volatile(
        Self::Reg::ADDRESS as *const RegFieldRegHoldValRaw<'a, T, Self>,
      ))
    }
  }
}

/// Register field that can write its value.
pub trait WRegField<'a, T>
where
  Self: RegField<'a, T>,
  Self::Reg: WReg<'a, T>,
  T: RegTag + 'a,
{
}

/// Register field that can only read its value.
pub trait RoRegField<'a, T>
where
  Self: RRegField<'a, T>,
  Self::Reg: RReg<'a, T>,
  T: RegTag + 'a,
{
}

/// Register field that can only write its value.
pub trait WoRegField<'a, T>
where
  Self: WRegField<'a, T>,
  Self::Reg: WReg<'a, T>,
  T: RegTag + 'a,
{
}

/// Write-only field of write-only register.
pub trait WoWoRegField<'a, T>
where
  Self: WoRegField<'a, T>,
  Self::Reg: WoReg<'a, T>,
  T: RegTag + 'a,
{
  /// Creates a new reset value.
  fn reset_val(&self) -> RegFieldRegHoldVal<'a, T, Self>;

  /// Writes the value `val`.
  fn store_val(&self, val: RegFieldRegHoldVal<'a, T, Self>);

  /// Updates a new reset value with `f` and writes the result to the register's
  /// memory address.
  fn reset<F>(&self, f: F)
  where
    F: Fn(&mut RegFieldRegHoldVal<'a, T, Self>);
}

/// Single-bit register field that can read its value.
pub trait RRegFieldBit<'a, T>
where
  Self: RegFieldBit<'a, T> + RRegField<'a, T>,
  Self::Reg: RReg<'a, T>,
  T: RegTag + 'a,
{
  /// Reads the state of the bit from `val`.
  fn read(&self, val: &RegFieldRegHoldVal<'a, T, Self>) -> bool;

  /// Reads the state of the bit from memory.
  fn read_bit(&self) -> bool;
}

/// Single-bit register field that can write its value.
pub trait WRegFieldBit<'a, T>
where
  Self: RegFieldBit<'a, T> + WRegField<'a, T>,
  Self::Reg: WReg<'a, T>,
  T: RegTag + 'a,
{
  /// Sets the bit in `val`.
  fn set(&self, val: &mut RegFieldRegHoldVal<'a, T, Self>);

  /// Clears the bit in `val`.
  fn clear(&self, val: &mut RegFieldRegHoldVal<'a, T, Self>);

  /// Toggles the bit in `val`.
  fn toggle(&self, val: &mut RegFieldRegHoldVal<'a, T, Self>);
}

/// Single-bit write-only field of write-only register.
pub trait WoWoRegFieldBit<'a, T>
where
  Self: RegFieldBit<'a, T> + WoRegField<'a, T>,
  Self::Reg: WoReg<'a, T>,
  T: RegTag + 'a,
{
  /// Sets the bit in memory.
  fn set_bit(&self);

  /// Clears the bit in memory.
  fn clear_bit(&self);

  /// Toggles the bit in memory.
  fn toggle_bit(&self);
}

/// Multiple-bits register field that can read its value.
pub trait RRegFieldBits<'a, T>
where
  Self: RegFieldBits<'a, T> + RRegField<'a, T>,
  Self::Reg: RReg<'a, T>,
  T: RegTag + 'a,
{
  /// Reads the bits from `val`.
  fn read(
    &self,
    val: &RegFieldRegHoldVal<'a, T, Self>,
  ) -> RegFieldRegHoldValRaw<'a, T, Self>;

  /// Reads the bits from memory.
  fn read_bits(&self) -> RegFieldRegHoldValRaw<'a, T, Self>;
}

/// Multiple-bits register field that can write its value.
pub trait WRegFieldBits<'a, T>
where
  Self: RegFieldBits<'a, T> + WRegField<'a, T>,
  Self::Reg: WReg<'a, T>,
  T: RegTag + 'a,
{
  /// Write `bits` to `val`.
  fn write(
    &self,
    val: &mut RegFieldRegHoldVal<'a, T, Self>,
    bits: RegFieldRegHoldValRaw<'a, T, Self>,
  );
}

/// Multiple-bits write-only field of write-only register.
pub trait WoWoRegFieldBits<'a, T>
where
  Self: RegFieldBits<'a, T> + WoRegField<'a, T>,
  Self::Reg: WoReg<'a, T>,
  T: RegTag + 'a,
{
  /// Sets the bit in memory.
  fn write_bits(&self, bits: RegFieldRegHoldValRaw<'a, T, Self>);
}

impl<'a, T, U> WoWoRegField<'a, T> for U
where
  T: RegTag + 'a,
  U: WoRegField<'a, T>,
  U::Reg: WoReg<'a, T>,
{
  #[inline]
  fn reset_val(&self) -> RegFieldRegHoldVal<'a, T, Self> {
    unsafe { RegFieldRegHoldVal::<'a, T, Self>::reset() }
  }

  #[inline]
  fn store_val(&self, val: RegFieldRegHoldVal<'a, T, Self>) {
    unsafe {
      write_volatile(
        Self::Reg::ADDRESS as *mut RegFieldRegHoldValRaw<'a, T, Self>,
        val.raw(),
      );
    }
  }

  #[inline]
  fn reset<F>(&self, f: F)
  where
    F: Fn(&mut RegFieldRegHoldVal<'a, T, Self>),
  {
    let mut val = self.reset_val();
    f(&mut val);
    self.store_val(val);
  }
}

impl<'a, T, U> RRegFieldBit<'a, T> for U
where
  T: RegTag + 'a,
  U: RegFieldBit<'a, T> + RRegField<'a, T>,
  U::Reg: RReg<'a, T>,
{
  #[inline]
  fn read(&self, val: &RegFieldRegHoldVal<'a, T, Self>) -> bool {
    unsafe {
      val.read_bit(RegFieldRegHoldValRaw::<'a, T, Self>::from_usize(
        Self::OFFSET,
      ))
    }
  }

  #[inline]
  fn read_bit(&self) -> bool {
    self.read(&self.load_val())
  }
}

impl<'a, T, U> WRegFieldBit<'a, T> for U
where
  T: RegTag + 'a,
  U: RegFieldBit<'a, T> + WRegField<'a, T>,
  U::Reg: WReg<'a, T>,
{
  #[inline]
  fn set(&self, val: &mut RegFieldRegHoldVal<'a, T, Self>) {
    unsafe {
      val.set_bit(RegFieldRegHoldValRaw::<'a, T, Self>::from_usize(
        Self::OFFSET,
      ));
    }
  }

  #[inline]
  fn clear(&self, val: &mut RegFieldRegHoldVal<'a, T, Self>) {
    unsafe {
      val.clear_bit(RegFieldRegHoldValRaw::<'a, T, Self>::from_usize(
        Self::OFFSET,
      ));
    }
  }

  #[inline]
  fn toggle(&self, val: &mut RegFieldRegHoldVal<'a, T, Self>) {
    unsafe {
      val.toggle_bit(RegFieldRegHoldValRaw::<'a, T, Self>::from_usize(
        Self::OFFSET,
      ));
    }
  }
}

impl<'a, T, U> WoWoRegFieldBit<'a, T> for U
where
  T: RegTag + 'a,
  U: RegFieldBit<'a, T> + WoRegField<'a, T>,
  U::Reg: WoReg<'a, T>,
{
  #[inline]
  fn set_bit(&self) {
    let mut val = self.reset_val();
    self.set(&mut val);
    self.store_val(val);
  }

  #[inline]
  fn clear_bit(&self) {
    let mut val = self.reset_val();
    self.clear(&mut val);
    self.store_val(val);
  }

  #[inline]
  fn toggle_bit(&self) {
    let mut val = self.reset_val();
    self.toggle(&mut val);
    self.store_val(val);
  }
}

impl<'a, T, U> RRegFieldBits<'a, T> for U
where
  T: RegTag + 'a,
  U: RegFieldBits<'a, T> + RRegField<'a, T>,
  U::Reg: RReg<'a, T>,
{
  #[inline]
  fn read(
    &self,
    val: &RegFieldRegHoldVal<'a, T, Self>,
  ) -> RegFieldRegHoldValRaw<'a, T, Self> {
    unsafe {
      val.read_bits(
        RegFieldRegHoldValRaw::<'a, T, Self>::from_usize(Self::OFFSET),
        RegFieldRegHoldValRaw::<'a, T, Self>::from_usize(Self::WIDTH),
      )
    }
  }

  #[inline]
  fn read_bits(&self) -> RegFieldRegHoldValRaw<'a, T, Self> {
    self.read(&self.load_val())
  }
}

impl<'a, T, U> WRegFieldBits<'a, T> for U
where
  T: RegTag + 'a,
  U: RegFieldBits<'a, T> + WRegField<'a, T>,
  U::Reg: WReg<'a, T>,
{
  #[inline]
  fn write(
    &self,
    val: &mut RegFieldRegHoldVal<'a, T, Self>,
    bits: RegFieldRegHoldValRaw<'a, T, Self>,
  ) {
    unsafe {
      val.write_bits(
        RegFieldRegHoldValRaw::<'a, T, Self>::from_usize(Self::OFFSET),
        RegFieldRegHoldValRaw::<'a, T, Self>::from_usize(Self::WIDTH),
        bits,
      );
    }
  }
}

impl<'a, T, U> WoWoRegFieldBits<'a, T> for U
where
  T: RegTag + 'a,
  U: RegFieldBits<'a, T> + WoRegField<'a, T>,
  U::Reg: WoReg<'a, T>,
{
  #[inline]
  fn write_bits(&self, bits: RegFieldRegHoldValRaw<'a, T, Self>) {
    let mut val = self.reset_val();
    self.write(&mut val, bits);
    self.store_val(val);
  }
}
