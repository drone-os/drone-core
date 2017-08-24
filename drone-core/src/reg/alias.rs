//! Register bit-banding aliases.

use core::ptr::{read_volatile, write_volatile};
use reg::{marker, RawBits};

const MASK: usize = 0xFF_FFF;

/// Register bit-band alias for a specific memory region.
pub trait RegionAlias<R> {
  /// Start of a bit-banding alias memory region.
  const BASE: usize;

  /// Constructs a new bit-band alias handler.
  ///
  /// # Safety
  ///
  /// Must be called only from a register pointer instance.
  unsafe fn new(address: usize) -> Self;

  /// Returns a bit-band alias base for the address.
  fn alias_base(address: usize) -> usize {
    Self::BASE + ((address & MASK) << 5)
  }
}

/// Base register bit-band alias.
pub trait RawAlias<R>: RegionAlias<R> {
  /// Returns a raw bit-band alias address of a first bit.
  fn get(&self) -> usize;

  /// Returns a corresponding pointer in a bit-band alias region.
  fn alias(&self, offset: usize) -> *mut u32 {
    (self.get() + (offset << 2)) as *mut u32
  }
}

impl<T, R> RawBits<R, marker::Alias> for T
where
  T: RawAlias<R>,
{
  fn write(&mut self, offset: u32, set: bool) -> &mut Self {
    assert!(offset < 0x20);
    let value = if set { 1 } else { 0 };
    unsafe {
      write_volatile(self.alias(offset as usize), value);
    }
    self
  }

  fn read(&self, offset: u32) -> bool {
    assert!(offset < 0x20);
    unsafe { read_volatile(self.alias(offset as usize)) != 0 }
  }
}
