use super::*;
use bitfield::Bitfield;
use core::ptr::{read_volatile, write_volatile};

/// Memory-mapped register token. Types which implement this trait should be
/// zero-sized. This is a zero-cost abstraction for safely working with
/// memory-mapped registers.
pub trait Reg<T: RegTag>: Sized + Send + Sync + 'static {
  /// Type that wraps a raw register value.
  type Val: Bitfield;

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
  #[allow(clippy::needless_lifetimes)]
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
    unsafe { Self::Val::from_bits(read_volatile(self.to_ptr())) }
  }

  /// Returns an unsafe constant pointer to the register's memory address.
  #[inline(always)]
  fn to_ptr(&self) -> *const <Self::Val as Bitfield>::Bits {
    Self::ADDRESS as *const <Self::Val as Bitfield>::Bits
  }
}

/// Register that can write its value.
pub trait WReg<T: RegTag>: Reg<T> {
  /// Returns an unsafe mutable pointer to the register's memory address.
  #[inline(always)]
  fn to_mut_ptr(&self) -> *mut <Self::Val as Bitfield>::Bits {
    Self::ADDRESS as *mut <Self::Val as Bitfield>::Bits
  }
}

/// Read-only register.
pub trait RoReg<T: RegTag>: RReg<T> {}

/// Write-only register.
pub trait WoReg<T: RegTag>: WReg<T> {}

/// Register that can write its value in a multi-threaded context.
// FIXME https://github.com/rust-lang/rust/issues/46397
pub trait WRegAtomic<'a, T: RegAtomic>: WReg<T> + RegRef<'a, T> {
  /// Updates a new reset value with `f` and writes the result to the register's
  /// memory address.
  fn store<F>(&'a self, f: F)
  where
    F: for<'b> FnOnce(&'b mut <Self as RegRef<'a, T>>::Hold)
      -> &'b mut <Self as RegRef<'a, T>>::Hold;

  /// Writes `val` into the register.
  fn store_val(&self, val: Self::Val);

  /// Writes the reset value to the register.
  fn reset(&'a self);
}

/// Register that can write its value in a single-threaded context.
// FIXME https://github.com/rust-lang/rust/issues/46397
pub trait WRegUnsync<'a>: WReg<Urt> + RegRef<'a, Urt> {
  /// Updates a new reset value with `f` and writes the result to the register's
  /// memory address.
  fn store<F>(&'a mut self, f: F)
  where
    F: for<'b> FnOnce(&'b mut <Self as RegRef<'a, Urt>>::Hold)
      -> &'b mut <Self as RegRef<'a, Urt>>::Hold;

  /// Writes `val` into the register.
  fn store_val(&mut self, val: Self::Val);

  /// Writes the reset value to the register.
  fn reset(&'a mut self);
}

/// Register that can read and write its value in a single-threaded context.
// FIXME https://github.com/rust-lang/rust/issues/46397
pub trait RwRegUnsync<'a>:
  RReg<Urt> + WRegUnsync<'a> + RegRef<'a, Urt>
{
  /// Atomically updates the register's value.
  fn modify<F>(&'a mut self, f: F)
  where
    F: for<'b> FnOnce(&'b mut <Self as RegRef<'a, Urt>>::Hold)
      -> &'b mut <Self as RegRef<'a, Urt>>::Hold;
}

impl<'a, T, U> WRegAtomic<'a, T> for U
where
  T: RegAtomic,
  U: WReg<T> + RegRef<'a, T>,
  // Extra bound to make the dot operator checking `WRegUnsync` first.
  U::Val: Bitfield,
{
  #[inline(always)]
  fn store<F>(&'a self, f: F)
  where
    F: for<'b> FnOnce(&'b mut <U as RegRef<'a, T>>::Hold)
      -> &'b mut <U as RegRef<'a, T>>::Hold,
  {
    self.store_val(f(&mut self.default()).val());
  }

  #[inline(always)]
  fn store_val(&self, val: U::Val) {
    unsafe { write_volatile(self.to_mut_ptr(), val.bits()) };
  }

  #[inline(always)]
  fn reset(&'a self) {
    self.store_val(self.default_val());
  }
}

impl<'a, T> WRegUnsync<'a> for T
where
  T: WReg<Urt> + RegRef<'a, Urt>,
{
  #[inline(always)]
  fn store<F>(&'a mut self, f: F)
  where
    F: for<'b> FnOnce(&'b mut <T as RegRef<'a, Urt>>::Hold)
      -> &'b mut <T as RegRef<'a, Urt>>::Hold,
  {
    unsafe {
      write_volatile(self.to_mut_ptr(), f(&mut self.default()).val().bits());
    }
  }

  #[inline(always)]
  fn store_val(&mut self, val: T::Val) {
    unsafe { write_volatile(self.to_mut_ptr(), val.bits()) };
  }

  #[inline(always)]
  fn reset(&'a mut self) {
    unsafe { write_volatile(self.to_mut_ptr(), self.default_val().bits()) };
  }
}

impl<'a, T> RwRegUnsync<'a> for T
where
  T: RReg<Urt> + WRegUnsync<'a> + RegRef<'a, Urt>,
{
  #[inline(always)]
  fn modify<F>(&'a mut self, f: F)
  where
    F: for<'b> FnOnce(&'b mut <T as RegRef<'a, Urt>>::Hold)
      -> &'b mut <T as RegRef<'a, Urt>>::Hold,
  {
    unsafe {
      write_volatile(self.to_mut_ptr(), f(&mut self.load()).val().bits());
    }
  }
}
