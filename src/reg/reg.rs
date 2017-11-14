use super::*;
use core::ptr::{read_volatile, write_volatile};

/// Disambiguation for `Reg::Hold::Val`
pub type RegHoldVal<'a, T, U> = <<U as Reg<'a, T>>::Hold as RegHold<
  'a,
  T,
  U,
>>::Val;

/// Disambiguation for `Reg::Hold::Val::Raw`
pub type RegHoldValRaw<'a, T, U> = <RegHoldVal<'a, T, U> as RegVal>::Raw;

/// Memory-mapped register binding. Types which implement this trait should be
/// zero-sized. This is a zero-cost abstraction for safely working with
/// memory-mapped registers.
pub trait Reg<'a, T>
where
  Self: Sized + 'a,
  T: RegTag + 'a,
{
  /// Type that wraps a raw register value and a register reference.
  type Hold: RegHold<'a, T, Self>;

  /// Type that owns register fields.
  type Fields: RegFields<'a, T, Self>;

  /// Memory address of the register.
  const ADDRESS: usize;

  /// Converts the register into the set of fields.
  fn into_fields(self) -> Self::Fields;

  /// Creates a new `Hold` for `val`.
  unsafe fn hold(&'a self, val: RegHoldVal<'a, T, Self>) -> Self::Hold {
    Self::Hold::hold(self, val)
  }

  /// Creates a new `Hold` with reset value.
  fn reset_val(&'a self) -> Self::Hold {
    unsafe { self.hold(RegHoldVal::<'a, T, Self>::reset()) }
  }
}

/// Register that can read its value.
pub trait RReg<'a, T>
where
  Self: Reg<'a, T>,
  T: RegTag + 'a,
{
  /// Reads and wraps a register value from its memory address.
  #[inline(always)]
  fn load(&'a self) -> Self::Hold {
    unsafe { self.hold(RegHoldVal::<'a, T, Self>::from_raw(self.load_raw())) }
  }

  /// Reads a raw register value from its memory address.
  #[inline(always)]
  unsafe fn load_raw(&self) -> RegHoldValRaw<'a, T, Self> {
    read_volatile(self.to_ptr())
  }

  /// Returns an unsafe constant pointer to the register's memory address.
  #[inline(always)]
  fn to_ptr(&self) -> *const RegHoldValRaw<'a, T, Self> {
    Self::ADDRESS as *const RegHoldValRaw<'a, T, Self>
  }
}

/// Register that can write its value.
pub trait WReg<'a, T>
where
  Self: Reg<'a, T>,
  T: RegTag + 'a,
{
  /// Updates a new reset value with `f` and writes the result to the register's
  /// memory address.
  #[inline(always)]
  fn reset<F>(&'a self, f: F)
  where
    F: FnOnce(&mut Self::Hold) -> &mut Self::Hold,
  {
    unsafe {
      self.store(f(&mut self.hold(RegHoldVal::<'a, T, Self>::reset())));
    }
  }

  /// Writes the holded value `val`.
  #[inline(always)]
  fn store(&self, val: &Self::Hold) {
    self.store_val(val.val());
  }

  /// Writes the unbound value `val`.
  #[inline(always)]
  fn store_val(&self, val: RegHoldVal<'a, T, Self>) {
    unsafe { self.store_raw(val.raw()) };
  }

  /// Writes a raw register value to its memory address.
  #[inline(always)]
  unsafe fn store_raw(&self, raw: RegHoldValRaw<'a, T, Self>) {
    write_volatile(self.to_mut_ptr(), raw);
  }

  /// Returns an unsafe mutable pointer to the register's memory address.
  #[inline(always)]
  fn to_mut_ptr(&self) -> *mut RegHoldValRaw<'a, T, Self> {
    Self::ADDRESS as *mut RegHoldValRaw<'a, T, Self>
  }
}

/// Read-only register.
pub trait RoReg<'a, T>
where
  Self: RReg<'a, T>,
  T: RegTag + 'a,
{
}

/// Write-only register.
pub trait WoReg<'a, T>
where
  Self: WReg<'a, T>,
  T: RegTag + 'a,
{
}

/// Register that can read and write its value in a single-threaded context.
pub trait RwRegUnique<'a>
where
  Self: RReg<'a, Ur> + WReg<'a, Ur>,
{
  /// Atomically updates a register's value.
  fn update<F>(&'a mut self, f: F)
  where
    F: FnOnce(&mut Self::Hold) -> &mut Self::Hold;
}

impl<'a, T> RwRegUnique<'a> for T
where
  T: RReg<'a, Ur> + WReg<'a, Ur>,
{
  #[inline(always)]
  fn update<F>(&'a mut self, f: F)
  where
    F: FnOnce(&mut Self::Hold) -> &mut Self::Hold,
  {
    self.store(f(&mut self.load()));
  }
}
