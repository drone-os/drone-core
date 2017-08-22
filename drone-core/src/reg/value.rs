//! Register value wrappers.


use reg::{RawBits, marker};


/// Base register value wrapper.
pub trait RawValue<R> {
  /// Constructs a new register value wrapper handler.
  fn new(value: u32) -> Self;


  /// Returns a raw register value.
  fn get(&self) -> u32;


  /// Replaces a raw register value.
  fn set(&mut self, value: u32) -> &mut Self;


  /// Copies any number of low order bits from a `source` into the same number
  /// of adjacent bits at any position in the stored register value.
  ///
  /// # Panics
  ///
  /// * If `offset` is greater or equals to `0x20`.
  /// * If `width + offset` is greater or equals to `0x20`.
  fn write_bits(&mut self, source: u32, width: u32, offset: u32) -> &mut Self {
    assert!(offset < 0x20);
    assert!(width < 0x20 - offset);
    let mask = (0b1 << width) - 1;
    let value = self.get() & !(mask << offset) | source << offset;
    self.set(value)
  }


  /// Reads any number of low order bits at any position from the stored
  /// register value.
  ///
  /// # Panics
  ///
  /// * If `offset` is greater or equals to `0x20`.
  /// * If `width + offset` is greater or equals to `0x20`.
  fn read_bits(&self, width: u32, offset: u32) -> u32 {
    assert!(offset < 0x20);
    assert!(width < 0x20 - offset);
    let mask = (0b1 << width) - 1;
    self.get() >> offset & mask
  }
}


impl<T, R> RawBits<R, marker::Value> for T
where
  T: RawValue<R>,
{
  fn write(&mut self, offset: u32, set: bool) -> &mut Self {
    assert!(offset < 0x20);
    let mask = 0b1 << offset;
    let value = self.get();
    self.set(if set { value | mask } else { value & !mask })
  }


  fn read(&self, offset: u32) -> bool {
    assert!(offset < 0x20);
    let mask = 0b1 << offset;
    self.get() & mask != 0
  }
}


#[cfg(test)]
mod tests {
  use super::*;


  struct TestValue(u32);


  impl RawValue<()> for TestValue {
    fn new(value: u32) -> TestValue {
      TestValue(value)
    }

    fn get(&self) -> u32 {
      self.0
    }

    fn set(&mut self, value: u32) -> &mut TestValue {
      self.0 = value;
      self
    }
  }


  #[test]
  fn write_bits() {
    let mut x = TestValue::new(0b0);
    assert_eq!(x.write_bits(0b0, 0, 0).get(), 0b0);
    let mut x = TestValue::new(0b0000_0000);
    assert_eq!(x.write_bits(0b1101, 4, 0).get(), 0b0000_1101);
    let mut x = TestValue::new(0b0000_0000);
    assert_eq!(x.write_bits(0b1101, 4, 3).get(), 0b0110_1000);
    let mut x = TestValue::new(0b1111_1111);
    assert_eq!(x.write_bits(0b0000, 2, 3).get(), 0b1110_0111);
    let mut x = TestValue::new(0b1111_1111);
    assert_eq!(x.write_bits(0b1011, 4, 4).get(), 0b1011_1111);
  }


  #[test]
  fn read_bits() {
    let x = TestValue::new(0b0000_0000);
    assert_eq!(x.read_bits(0, 0), 0b0000);
    let x = TestValue::new(0b0010_0100);
    assert_eq!(x.read_bits(4, 2), 0b1001);
    let x = TestValue::new(0b1010_0000);
    assert_eq!(x.read_bits(3, 5), 0b0101);
  }
}
