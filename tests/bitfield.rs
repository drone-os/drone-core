#![no_implicit_prelude]

use ::drone_core::bitfield::Bitfield;
use ::std::assert_eq;

#[derive(Bitfield, Copy, Clone)]
#[bitfield(
    foo(rw, 0, 1, "Test read-write bit."),
    bar(r, 1, 2, "Test read-only bits."),
    baz(w, 3, 3, "Test write-only bits.")
)]
pub struct Byte(u8);

#[test]
fn read_bit() {
    let x = Byte(0b1010_1010);
    assert!(!unsafe { x.read_bit(0) });
    assert!(unsafe { x.read_bit(1) });
    assert!(!unsafe { x.read_bit(2) });
    assert!(unsafe { x.read_bit(3) });
    assert!(!unsafe { x.read_bit(4) });
    assert!(unsafe { x.read_bit(5) });
    assert!(!unsafe { x.read_bit(6) });
    assert!(unsafe { x.read_bit(7) });
}

#[test]
fn set_bit() {
    let mut x = Byte(0b1010_1010);
    unsafe {
        x.set_bit(0);
        x.set_bit(7);
        x.set_bit(4);
        x.set_bit(3);
    }
    assert_eq!(x.bits(), 0b1011_1011);
}

#[test]
fn clear_bit() {
    let mut x = Byte(0b1010_1010);
    unsafe {
        x.clear_bit(0);
        x.clear_bit(7);
        x.clear_bit(4);
        x.clear_bit(3);
    }
    assert_eq!(x.bits(), 0b0010_0010);
}

#[test]
fn toggle_bit() {
    let mut x = Byte(0b1010_1010);
    unsafe {
        x.toggle_bit(0);
        x.toggle_bit(7);
        x.toggle_bit(4);
        x.toggle_bit(3);
    }
    assert_eq!(x.bits(), 0b0011_0011);
}

#[test]
fn write_bit() {
    let mut x = Byte(0b1010_1010);
    unsafe {
        x.write_bit(0, true);
        x.write_bit(7, false);
        x.write_bit(4, false);
        x.write_bit(3, true);
    }
    assert_eq!(x.bits(), 0b0010_1011);
}

#[test]
fn read_bits() {
    let x = Byte(0b1010_0110);
    assert_eq!(unsafe { x.read_bits(0, 0) }, 0b0);
    assert_eq!(unsafe { x.read_bits(1, 0) }, 0b0);
    assert_eq!(unsafe { x.read_bits(2, 0) }, 0b0);
    assert_eq!(unsafe { x.read_bits(0, 1) }, 0b0);
    assert_eq!(unsafe { x.read_bits(1, 1) }, 0b1);
    assert_eq!(unsafe { x.read_bits(2, 1) }, 0b1);
    assert_eq!(unsafe { x.read_bits(3, 1) }, 0b0);
    assert_eq!(unsafe { x.read_bits(4, 1) }, 0b0);
    assert_eq!(unsafe { x.read_bits(5, 1) }, 0b1);
    assert_eq!(unsafe { x.read_bits(6, 1) }, 0b0);
    assert_eq!(unsafe { x.read_bits(7, 1) }, 0b1);
    assert_eq!(unsafe { x.read_bits(0, 4) }, 0b0110);
    assert_eq!(unsafe { x.read_bits(1, 4) }, 0b0011);
    assert_eq!(unsafe { x.read_bits(2, 4) }, 0b1001);
    assert_eq!(unsafe { x.read_bits(3, 4) }, 0b0100);
    assert_eq!(unsafe { x.read_bits(4, 4) }, 0b1010);
    assert_eq!(unsafe { x.read_bits(0, 7) }, 0b0100_110);
    assert_eq!(unsafe { x.read_bits(1, 7) }, 0b1010_011);
    assert_eq!(unsafe { x.read_bits(0, 8) }, 0b1010_0110);
}

#[test]
fn write_bits() {
    let mut x = Byte(0b1010_0110);
    unsafe { x.write_bits(0, 0, 0b0) };
    unsafe { x.write_bits(1, 0, 0b1) };
    unsafe { x.write_bits(6, 0, 0b11) };
    unsafe { x.write_bits(7, 0, 0b111) };
    assert_eq!(x.bits(), 0b1010_0110);
    let mut x = Byte(0b1010_0110);
    unsafe { x.write_bits(0, 1, 0b1) };
    unsafe { x.write_bits(1, 1, 0b0) };
    unsafe { x.write_bits(7, 1, 0b0) };
    unsafe { x.write_bits(5, 1, 0b1) };
    assert_eq!(x.bits(), 0b0010_0101);
    let mut x = Byte(0b1010_0110);
    unsafe { x.write_bits(0, 4, 0b1001) };
    assert_eq!(x.bits(), 0b1010_1001);
    unsafe { x.write_bits(1, 4, 0b1001) };
    assert_eq!(x.bits(), 0b1011_0011);
    unsafe { x.write_bits(2, 4, 0b1001) };
    assert_eq!(x.bits(), 0b1010_0111);
    unsafe { x.write_bits(3, 4, 0b1001) };
    assert_eq!(x.bits(), 0b1100_1111);
    unsafe { x.write_bits(4, 4, 0b1001) };
    assert_eq!(x.bits(), 0b1001_1111);
    let mut x = Byte(0b1010_0110);
    unsafe { x.write_bits(0, 7, 0b1010_011) };
    assert_eq!(x.bits(), 0b1101_0011);
    unsafe { x.write_bits(1, 7, 0b1010_011) };
    assert_eq!(x.bits(), 0b1010_0111);
    let mut x = Byte(0b0001_1000);
    unsafe { x.write_bits(0, 8, 0b1111_1111) };
    assert_eq!(x.bits(), 0b1111_1111);
}
