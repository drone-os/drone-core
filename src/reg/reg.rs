use super::*;
use core::ptr::{read_volatile, write_volatile};

/// Memory-mapped register binding. Types which implement this trait should be
/// zero-sized. This is a zero-cost abstraction for safely working with
/// memory-mapped registers.
pub trait Reg<T>
where
  Self: Sized,
  T: RegTag,
{
  /// Type that wraps a raw register value.
  type Val: RegVal;

  /// Memory address of the register.
  const ADDRESS: usize;
}

/// Referenceable register.
pub trait RegRef<'a, T>
where
  Self: Reg<T>,
  T: RegTag,
{
  /// Type that wraps a raw register value and a register reference.
  type Hold: RegHold<'a, T, Self>;

  /// Creates a new `Hold` for `val`.
  fn hold(&'a self, val: Self::Val) -> Self::Hold {
    unsafe { Self::Hold::new(self, val) }
  }

  /// Creates a new `Hold` with reset value.
  fn default(&'a self) -> Self::Hold {
    unsafe { self.hold(Self::Val::reset()) }
  }
}

/// Unique register.
pub trait UReg
where
  Self: Reg<Urt>,
{
  /// Less strict type.
  type UpReg: Reg<Srt>;

  /// Converts to a less strict type.
  fn upgrade(self) -> Self::UpReg;
}

/// Synchronous register.
pub trait SReg
where
  Self: Reg<Srt>,
{
  /// Less strict type.
  type UpReg: Reg<Drt>;

  /// Converts to a less strict type.
  fn upgrade(self) -> Self::UpReg;
}

/// Duplicable register.
pub trait DReg
where
  Self: Reg<Drt>,
{
  /// Less strict type.
  type UpReg: Reg<Crt>;

  /// Converts to a less strict type.
  fn upgrade(self) -> Self::UpReg;

  /// Returns a copy of the register.
  fn clone(&mut self) -> Self;
}

/// Register that can read its value.
pub trait RReg<T>
where
  Self: Reg<T>,
  T: RegTag,
{
  /// Reads and wraps a register value from its memory address.
  #[inline(always)]
  #[cfg_attr(feature = "clippy", allow(needless_lifetimes))]
  fn load<'a>(&'a self) -> <Self as RegRef<'a, T>>::Hold
  where
    Self: RegRef<'a, T>,
  {
    unsafe { self.hold(Self::Val::from_raw(self.load_raw())) }
  }

  /// Reads a raw register value from its memory address.
  #[inline(always)]
  unsafe fn load_raw(&self) -> <Self::Val as RegVal>::Raw {
    read_volatile(self.to_ptr())
  }

  /// Returns an unsafe constant pointer to the register's memory address.
  #[inline(always)]
  fn to_ptr(&self) -> *const <Self::Val as RegVal>::Raw {
    Self::ADDRESS as *const <Self::Val as RegVal>::Raw
  }
}

/// Register that can write its value.
pub trait WReg<T>
where
  Self: Reg<T>,
  T: RegTag,
{
  /// Returns an unsafe mutable pointer to the register's memory address.
  #[inline(always)]
  fn to_mut_ptr(&self) -> *mut <Self::Val as RegVal>::Raw {
    Self::ADDRESS as *mut <Self::Val as RegVal>::Raw
  }

  /// Writes a raw register value to its memory address.
  #[inline(always)]
  unsafe fn store_raw(&self, raw: <Self::Val as RegVal>::Raw) {
    write_volatile(self.to_mut_ptr(), raw);
  }
}

/// Read-only register.
pub trait RoReg<T>
where
  Self: RReg<T>,
  T: RegTag,
{
}

/// Write-only register.
pub trait WoReg<T>
where
  Self: WReg<T>,
  T: RegTag,
{
}

/// Register that can write its value in a multi-threaded context.
// FIXME https://github.com/rust-lang/rust/issues/46397
pub trait WRegShared<'a, T>
where
  Self: WReg<T> + RegRef<'a, T>,
  T: RegShared,
{
  /// Updates a new reset value with `f` and writes the result to the register's
  /// memory address.
  fn reset<F>(&'a self, f: F)
  where
    F: for<'b> FnOnce(&'b mut <Self as RegRef<'a, T>>::Hold)
      -> &'b mut <Self as RegRef<'a, T>>::Hold;

  /// Writes `val` into the register.
  fn store_val(&self, val: Self::Val);
}

/// Register that can write its value in a single-threaded context.
// FIXME https://github.com/rust-lang/rust/issues/46397
pub trait WRegUnique<'a>
where
  Self: WReg<Urt> + RegRef<'a, Urt>,
{
  /// Updates a new reset value with `f` and writes the result to the register's
  /// memory address.
  fn reset<F>(&'a mut self, f: F)
  where
    F: for<'b> FnOnce(&'b mut <Self as RegRef<'a, Urt>>::Hold)
      -> &'b mut <Self as RegRef<'a, Urt>>::Hold;

  /// Writes `val` into the register.
  fn store_val(&mut self, val: Self::Val);
}

/// Register that can read and write its value in a single-threaded context.
// FIXME https://github.com/rust-lang/rust/issues/46397
pub trait RwRegUnique<'a>
where
  Self: RReg<Urt> + WRegUnique<'a> + RegRef<'a, Urt>,
{
  /// Atomically updates the register's value.
  fn update<F>(&'a mut self, f: F)
  where
    F: for<'b> FnOnce(&'b mut <Self as RegRef<'a, Urt>>::Hold)
      -> &'b mut <Self as RegRef<'a, Urt>>::Hold;
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
    unsafe { self.store_raw(val.raw()) };
  }
}

impl<'a, T> WRegUnique<'a> for T
where
  T: WReg<Urt> + RegRef<'a, Urt>,
{
  #[inline(always)]
  fn reset<F>(&'a mut self, f: F)
  where
    F: for<'b> FnOnce(&'b mut <T as RegRef<'a, Urt>>::Hold)
      -> &'b mut <T as RegRef<'a, Urt>>::Hold,
  {
    unsafe { self.store_raw(f(&mut self.default()).val().raw()) };
  }

  #[inline(always)]
  fn store_val(&mut self, val: T::Val) {
    unsafe { self.store_raw(val.raw()) };
  }
}

impl<'a, T> RwRegUnique<'a> for T
where
  T: RReg<Urt> + WRegUnique<'a> + RegRef<'a, Urt>,
{
  #[inline(always)]
  fn update<F>(&'a mut self, f: F)
  where
    F: for<'b> FnOnce(&'b mut <T as RegRef<'a, Urt>>::Hold)
      -> &'b mut <T as RegRef<'a, Urt>>::Hold,
  {
    unsafe { self.store_raw(f(&mut self.load()).val().raw()) };
  }
}
