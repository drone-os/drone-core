//! ITM stimulus ports functionality.

use core::fmt::{self, Write};
use core::slice;

const ADDRESS_BASE: usize = 0xE000_0000;

/// ITM stimulus port pointer.
pub struct Port(usize);

/// Types that can be transmitted through ITM stimulus port.
pub trait Transmit: Copy {
  /// Writes its value to provided address of a stimulus port register.
  ///
  /// It retries on buffer overflow.
  fn transmit(self, address: usize);
}

impl Port {
  /// Constructs a new `Port`.
  ///
  /// # Panics
  ///
  /// If `port` is greater or equals to `0x20`.
  pub fn new(port: usize) -> Port {
    assert!(port < 0x20);
    Port(ADDRESS_BASE + port)
  }

  /// Writes a buffer in most effective chunks, splitting it to 8- and 32-bit
  /// slices.
  pub fn write_stream(&self, buffer: &[u8]) {
    let mut end = buffer.len();
    if end < 4 {
      return self.write_all(buffer);
    }
    let mut start = buffer.as_ptr() as usize;
    let mut rem = start & 0b11;
    end += start;
    if rem != 0 {
      rem = 0b100 - rem;
      self.write_all(unsafe { slice::from_raw_parts(start as *const u8, rem) });
      start += rem;
    }
    rem = end & 0b11;
    end -= rem;
    self.write_all(unsafe {
      slice::from_raw_parts(start as *const u32, end - start >> 2)
    });
    self.write_all(unsafe { slice::from_raw_parts(end as *const u8, rem) });
  }

  /// Writes an entire buffer in chunks of `size_of::<T>() * 8` bits.
  pub fn write_all<T: Transmit>(&self, buffer: &[T]) {
    for item in buffer {
      self.write(*item);
    }
  }

  /// Writes a value into a port.
  pub fn write<T: Transmit>(&self, value: T) {
    value.transmit(self.0);
  }
}

impl Write for Port {
  fn write_str(&mut self, string: &str) -> fmt::Result {
    self.write_stream(string.as_bytes());
    Ok(())
  }
}

impl Transmit for u8 {
  fn transmit(self, address: usize) {
    unsafe {
      asm!("
        0:
          ldrexb r0, [$1]
          cmp r0, #0
          itt ne
          strexbne r0, $0, [$1]
          cmpne r0, #1
          beq 0b
      " :
        : "r"(self), "r"(address as *mut u8)
        : "r0", "cc"
        : "volatile");
    }
  }
}

impl Transmit for u16 {
  fn transmit(self, address: usize) {
    unsafe {
      asm!("
        0:
          ldrexh r0, [$1]
          cmp r0, #0
          itt ne
          strexhne r0, $0, [$1]
          cmpne r0, #1
          beq 0b
      " :
        : "r"(self), "r"(address as *mut u16)
        : "r0", "cc"
        : "volatile");
    }
  }
}

impl Transmit for u32 {
  fn transmit(self, address: usize) {
    unsafe {
      asm!("
        0:
          ldrex r0, [$1]
          cmp r0, #0
          itt ne
          strexne r0, $0, [$1]
          cmpne r0, #1
          beq 0b
      " :
        : "r"(self), "r"(address as *mut u32)
        : "r0", "cc"
        : "volatile");
    }
  }
}
