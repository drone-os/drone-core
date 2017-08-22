//! Memory-mapped registers.


pub use self::alias::*;
pub use self::pointer::*;
pub use self::value::*;


#[macro_use]
pub mod macros;
pub mod alias;
pub mod pointer;
pub mod value;


/// Primitive types representing properties of register types.
pub mod marker {
  /// Thread-unsafe marker.
  pub struct Single;


  /// Thread-safe marker.
  pub struct Atomic;


  /// A value type marker.
  pub struct Value;


  /// A bit-band alias type marker.
  pub struct Alias;
}


/// Types, that can write to distinct bits.
pub trait RawBits<R, T> {
  /// Sets or clears a bit by `offset`.
  ///
  /// # Panics
  ///
  /// If `offset` is greater or equals to `0x20`.
  fn write(&mut self, offset: u32, set: bool) -> &mut Self;


  /// Checks that a bit by `offset` is set.
  ///
  /// # Panics
  ///
  /// If `offset` is greater or equals to `0x20`.
  fn read(&self, offset: u32) -> bool;
}


/// Register delegate.
pub trait Delegate<R, A> {
  /// A corresponding register pointer type.
  type Pointer: RawPointer<R, A>;


  /// An address of the register in the memory.
  const ADDRESS: usize;


  /// Returns a new register pointer.
  fn ptr(&self) -> Self::Pointer {
    unsafe { Self::Pointer::new(Self::ADDRESS) }
  }
}
