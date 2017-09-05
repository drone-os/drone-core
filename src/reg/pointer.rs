//! Register pointers.

use core::ptr::{read_volatile, write_volatile};
use reg::{marker, RawAlias, RawValue, RegionAlias};

/// Base register pointer.
pub trait RawPointer<R, A> {
  /// Constructs a new register pointer handler.
  ///
  /// # Safety
  ///
  /// Must be called only by the register delegate.
  unsafe fn new(address: usize) -> Self;

  /// Returns a raw register address.
  fn get(&self) -> usize;

  /// Reads a raw value at the register address.
  fn read_raw(&self) -> u32 {
    unsafe { read_volatile(self.get() as *const u32) }
  }

  /// Writes a raw value at the register address.
  fn write_raw(&self, value: u32) {
    unsafe {
      write_volatile(self.get() as *mut u32, value);
    }
  }
}

/// A multi-thread pointer.
pub trait ThreadPointer<R, A>: RawPointer<R, A> {
  /// Reads, modifies, and writes a raw value at the register address.
  fn modify_raw<F>(&self, f: F)
  where
    F: Fn(u32) -> u32;
}

/// Register pointer with an associated value wrapper.
pub trait ValuePointer<R, A>: ThreadPointer<R, A> {
  /// A corresponding register value wrapper type.
  type Value: RawValue<R>;

  /// Reads a wrapped value at the register address.
  fn read(&self) -> Self::Value {
    Self::Value::new(self.read_raw())
  }

  /// Writes a wrapped value at the register address.
  fn write<F>(&self, f: F)
  where
    F: Fn(&mut Self::Value) -> &Self::Value,
  {
    self.write_raw(f(&mut Self::Value::new(0)).get());
  }

  /// Reads, modifies, and writes a wrapped value at the register address.
  fn modify<F>(&self, f: F)
  where
    F: Fn(&mut Self::Value) -> &Self::Value,
  {
    self.modify_raw(|x| f(&mut Self::Value::new(x)).get());
  }
}

/// Register pointer with an associated bit-band alias.
pub trait AliasPointer<R, A>: RawPointer<R, A> {
  /// A corresponding register bit-band alias type.
  type Alias: RawAlias<R>;

  /// Returns a register bit-band alias.
  fn bits(&self) -> Self::Alias {
    unsafe { Self::Alias::new(self.get()) }
  }
}

// By default pointers supposed to be atomic.
impl<T, R, A> ThreadPointer<R, A> for T
where
  T: RawPointer<R, A>,
{
  default fn modify_raw<F>(&self, f: F)
  where
    F: Fn(u32) -> u32,
  {
    let address = self.get() as *mut u32;
    let mut value: u32;
    let mut status: u32;
    loop {
      unsafe {
        asm!("
          ldrex $0, [$1]
        " : "=r"(value)
          : "r"(address)
          :
          : "volatile");
      }
      value = f(value);
      unsafe {
        asm!("
          strex $0, $1, [$2]
        " : "=r"(status)
          : "r"(value), "r"(address)
          :
          : "volatile");
      }
      if status == 0 {
        break;
      }
    }
  }
}

// Specialization for single pointer. Make it thread-unsafe.
impl<T, R> ThreadPointer<R, marker::Single> for T
where
  T: RawPointer<R, marker::Single>,
{
  fn modify_raw<F>(&self, f: F)
  where
    F: Fn(u32) -> u32,
  {
    self.write_raw(f(self.read_raw()));
  }
}
