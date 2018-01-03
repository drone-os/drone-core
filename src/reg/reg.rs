use super::*;
use core::ptr::{read_volatile, write_volatile};

/// Memory-mapped register token. Types which implement this trait should be
/// zero-sized. This is a zero-cost abstraction for safely working with
/// memory-mapped registers.
pub trait Reg<T: RegTag>: Sized {
  /// Type that wraps a raw register value.
  type Val: RegVal;

  /// Memory address of the register.
  const ADDRESS: usize;
}

/// Referenceable register.
pub trait RegRef<'a, T: RegTag>: Reg<T> {
  /// Type that wraps a raw register value and a register reference.
  type Hold: RegHold<'a, T, Self>;

  /// Creates a new `Hold` for `val`.
  #[inline(always)]
  fn hold(&'a self, val: Self::Val) -> Self::Hold {
    unsafe { Self::Hold::new(self, val) }
  }

  /// Creates a new `Hold` with reset value.
  #[inline(always)]
  fn default(&'a self) -> Self::Hold {
    self.hold(self.default_val())
  }

  /// Returns a default value.
  #[inline(always)]
  fn default_val(&self) -> Self::Val {
    unsafe { Self::Val::default() }
  }
}

/// Register that can read its value.
pub trait RReg<T: RegTag>: Reg<T> {
  /// Reads and wraps a register value from its memory address.
  #[cfg_attr(feature = "clippy", allow(needless_lifetimes))]
  #[inline(always)]
  fn load<'a>(&'a self) -> <Self as RegRef<'a, T>>::Hold
  where
    Self: RegRef<'a, T>,
  {
    self.hold(self.load_val())
  }

  /// Reads a register value from its memory address.
  #[inline(always)]
  fn load_val(&self) -> Self::Val {
    unsafe { Self::Val::from_raw(read_volatile(self.to_ptr())) }
  }

  /// Returns an unsafe constant pointer to the register's memory address.
  #[inline(always)]
  fn to_ptr(&self) -> *const <Self::Val as RegVal>::Raw {
    Self::ADDRESS as *const <Self::Val as RegVal>::Raw
  }
}

/// Register that can write its value.
pub trait WReg<T: RegTag>: Reg<T> {
  /// Returns an unsafe mutable pointer to the register's memory address.
  #[inline(always)]
  fn to_mut_ptr(&self) -> *mut <Self::Val as RegVal>::Raw {
    Self::ADDRESS as *mut <Self::Val as RegVal>::Raw
  }
}

/// Read-only register.
pub trait RoReg<T: RegTag>: RReg<T> {}

/// Write-only register.
pub trait WoReg<T: RegTag>: WReg<T> {}

/// Register that can write its value in a multi-threaded context.
// FIXME https://github.com/rust-lang/rust/issues/46397
pub trait WRegShared<'a, T: RegShared>: WReg<T> + RegRef<'a, T> {
  /// Updates a new reset value with `f` and writes the result to the register's
  /// memory address.
  fn reset<F>(&'a self, f: F)
  where
    F: for<'b> FnOnce(&'b mut <Self as RegRef<'a, T>>::Hold)
      -> &'b mut <Self as RegRef<'a, T>>::Hold;

  /// Writes `val` into the register.
  fn store_val(&self, val: Self::Val);

  /// Writes the reset value to the register.
  fn store_default(&'a self);
}

/// Register that can write its value in a single-threaded context.
// FIXME https://github.com/rust-lang/rust/issues/46397
pub trait WRegUnique<'a>: WReg<Utt> + RegRef<'a, Utt> {
  /// Updates a new reset value with `f` and writes the result to the register's
  /// memory address.
  fn reset<F>(&'a mut self, f: F)
  where
    F: for<'b> FnOnce(&'b mut <Self as RegRef<'a, Utt>>::Hold)
      -> &'b mut <Self as RegRef<'a, Utt>>::Hold;

  /// Writes `val` into the register.
  fn store_val(&mut self, val: Self::Val);

  /// Writes the reset value to the register.
  fn store_default(&'a mut self);
}

/// Register that can read and write its value in a single-threaded context.
// FIXME https://github.com/rust-lang/rust/issues/46397
pub trait RwRegUnique<'a>: RReg<Utt> + WRegUnique<'a> + RegRef<'a, Utt> {
  /// Atomically updates the register's value.
  fn modify<F>(&'a mut self, f: F)
  where
    F: for<'b> FnOnce(&'b mut <Self as RegRef<'a, Utt>>::Hold)
      -> &'b mut <Self as RegRef<'a, Utt>>::Hold;
}

impl<'a, T, U> WRegShared<'a, T> for U
where
  T: RegShared,
  U: WReg<T> + RegRef<'a, T>,
  // Extra bound to make the dot operator checking `WRegUnique` first.
  U::Val: RegVal,
{
  #[inline(always)]
  fn reset<F>(&'a self, f: F)
  where
    F: for<'b> FnOnce(&'b mut <U as RegRef<'a, T>>::Hold)
      -> &'b mut <U as RegRef<'a, T>>::Hold,
  {
    self.store_val(f(&mut self.default()).val());
  }

  #[inline(always)]
  fn store_val(&self, val: U::Val) {
    unsafe { write_volatile(self.to_mut_ptr(), val.raw()) };
  }

  #[inline(always)]
  fn store_default(&'a self) {
    self.store_val(self.default_val());
  }
}

impl<'a, T> WRegUnique<'a> for T
where
  T: WReg<Utt> + RegRef<'a, Utt>,
{
  #[inline(always)]
  fn reset<F>(&'a mut self, f: F)
  where
    F: for<'b> FnOnce(&'b mut <T as RegRef<'a, Utt>>::Hold)
      -> &'b mut <T as RegRef<'a, Utt>>::Hold,
  {
    unsafe {
      write_volatile(self.to_mut_ptr(), f(&mut self.default()).val().raw());
    }
  }

  #[inline(always)]
  fn store_val(&mut self, val: T::Val) {
    unsafe { write_volatile(self.to_mut_ptr(), val.raw()) };
  }

  #[inline(always)]
  fn store_default(&'a mut self) {
    unsafe { write_volatile(self.to_mut_ptr(), self.default_val().raw()) };
  }
}

impl<'a, T> RwRegUnique<'a> for T
where
  T: RReg<Utt> + WRegUnique<'a> + RegRef<'a, Utt>,
{
  #[inline(always)]
  fn modify<F>(&'a mut self, f: F)
  where
    F: for<'b> FnOnce(&'b mut <T as RegRef<'a, Utt>>::Hold)
      -> &'b mut <T as RegRef<'a, Utt>>::Hold,
  {
    unsafe {
      write_volatile(self.to_mut_ptr(), f(&mut self.load()).val().raw());
    }
  }
}
