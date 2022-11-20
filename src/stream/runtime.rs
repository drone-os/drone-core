#![cfg_attr(feature = "host", allow(unused_imports, unused_variables))]

use crate::platform::Interrupts;
use core::ptr;
use drone_stream::{GlobalRuntime, Runtime, HEADER_LENGTH};

const DEFAULT_TRANSACTION_LENGTH: u8 = 64;

pub trait LocalGlobalRuntime {
    fn is_enabled(&self, stream: u8) -> bool;
}

pub trait LocalRuntime {
    unsafe fn write_bytes(&mut self, stream: u8, buffer: *const u8, length: usize);

    unsafe fn write_transaction(&mut self, stream: u8, buffer: *const u8, length: u8);
}

impl LocalGlobalRuntime for GlobalRuntime {
    fn is_enabled(&self, stream: u8) -> bool {
        unsafe { ptr::addr_of!(self.enable_mask).read_volatile() & 1 << stream != 0 }
    }
}

impl LocalRuntime for Runtime {
    #[inline(never)]
    #[export_name = "stream_write_bytes"]
    unsafe fn write_bytes(&mut self, stream: u8, mut buffer: *const u8, mut length: usize) {
        while length > usize::from(DEFAULT_TRANSACTION_LENGTH) {
            length -= usize::from(DEFAULT_TRANSACTION_LENGTH);
            unsafe { self.write_transaction(stream, buffer, DEFAULT_TRANSACTION_LENGTH) };
            buffer = unsafe { buffer.add(usize::from(DEFAULT_TRANSACTION_LENGTH)) };
        }
        if length > 0 {
            unsafe { self.write_transaction(stream, buffer, length as u8) };
        }
    }

    #[inline(never)]
    #[export_name = "stream_write_transaction"]
    unsafe fn write_transaction(&mut self, stream: u8, buffer: *const u8, length: u8) {
        #[cfg(feature = "host")]
        return unimplemented!();
        #[cfg(not(feature = "host"))]
        loop {
            let complete = Interrupts::paused(|| unsafe {
                let transaction = Transaction {
                    buffer: ptr::addr_of_mut!(*self).add(1).cast::<u8>(),
                    buffer_size: self.buffer_size,
                    write_cursor: ptr::addr_of_mut!(self.write_cursor),
                    read_cursor: ptr::addr_of!(self.read_cursor),
                    stream,
                    source: buffer,
                    source_size: length,
                };
                transaction.write()
            });
            if complete {
                break;
            }
        }
    }
}

struct Transaction {
    buffer: *mut u8,
    buffer_size: u32,
    write_cursor: *mut u32,
    read_cursor: *const u32,
    stream: u8,
    source: *const u8,
    source_size: u8,
}

impl Transaction {
    unsafe fn write(self) -> bool {
        unsafe {
            let read_cursor = self.read_cursor.read_volatile();
            let write_cursor = self.write_cursor.read_volatile();
            let wrapped = write_cursor >= read_cursor;
            let available = if wrapped { self.buffer_size } else { read_cursor } - write_cursor;
            let frame_length = u32::from(self.source_size) + HEADER_LENGTH;
            let cursor = self.buffer.add(write_cursor as usize);
            if available >= frame_length {
                let mut next_write_cursor = write_cursor + frame_length;
                if next_write_cursor == self.buffer_size {
                    next_write_cursor = 0;
                }
                if next_write_cursor == read_cursor {
                    return false;
                }
                *cursor = self.stream;
                *cursor.add(1) = self.source_size;
                cursor.add(2).copy_from_nonoverlapping(self.source, usize::from(self.source_size));
                self.write_cursor.write_volatile(next_write_cursor);
                return true;
            }
            if wrapped && read_cursor != 0 {
                *cursor = 0xFF;
                self.write_cursor.write_volatile(0);
            }
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Runtime {
        write_cursor: u32,
        read_cursor: u32,
        buffer: Vec<u8>,
    }

    impl Runtime {
        fn new(buffer: &[u8]) -> Self {
            Self { write_cursor: 0, read_cursor: 0, buffer: buffer.to_vec() }
        }

        fn write(&mut self, stream: u8, source: &[u8]) -> bool {
            let transaction = Transaction {
                buffer: self.buffer.as_mut_ptr(),
                buffer_size: self.buffer.len() as u32,
                write_cursor: &mut self.write_cursor,
                read_cursor: &self.read_cursor,
                stream,
                source: source.as_ptr(),
                source_size: source.len() as u8,
            };
            unsafe { transaction.write() }
        }
    }

    #[test]
    fn test_biggest() {
        let mut runtime = Runtime::new(&[0; 8]);
        assert!(runtime.write(42, b"hello"));
        assert_eq!(&runtime.buffer[2..7], b"hello");
        assert_eq!(runtime.write_cursor, 7);
        assert_eq!(runtime.buffer[0], 42);
        assert_eq!(runtime.buffer[1], 5);
    }

    #[test]
    fn test_overflow() {
        let mut runtime = Runtime::new(&[0; 7]);
        assert!(!runtime.write(0, b"hello"));
    }

    #[test]
    fn test_to_the_end() {
        let mut runtime = Runtime::new(&[0; 11]);
        runtime.write_cursor = 4;
        runtime.read_cursor = 4;
        assert!(runtime.write(42, b"hello"));
        assert_eq!(&runtime.buffer[6..11], b"hello");
        assert_eq!(runtime.write_cursor, 0);
        assert_eq!(runtime.buffer[4], 42);
        assert_eq!(runtime.buffer[5], 5);
    }

    #[test]
    fn test_to_the_end_overflow() {
        let mut runtime = Runtime::new(&[0; 11]);
        runtime.write_cursor = 4;
        runtime.read_cursor = 0;
        assert!(!runtime.write(42, b"hello"));
    }

    #[test]
    fn test_in_the_middle() {
        let mut runtime = Runtime::new(&[0; 11]);
        runtime.write_cursor = 2;
        runtime.read_cursor = 10;
        assert!(runtime.write(42, b"hello"));
        assert_eq!(&runtime.buffer[4..9], b"hello");
        assert_eq!(runtime.write_cursor, 9);
        assert_eq!(runtime.buffer[2], 42);
        assert_eq!(runtime.buffer[3], 5);
    }

    #[test]
    fn test_wrap() {
        let mut runtime = Runtime::new(&[0; 8]);
        runtime.write_cursor = 2;
        runtime.read_cursor = 2;
        assert!(!runtime.write(42, b"hello"));
        assert_eq!(runtime.write_cursor, 0);
        assert_eq!(runtime.buffer[2], 0xFF);
    }

    #[test]
    fn test_wrap_when_read_cursor_zero() {
        let mut runtime = Runtime::new(&[0; 8]);
        runtime.write_cursor = 2;
        runtime.read_cursor = 0;
        assert!(!runtime.write(42, b"hello"));
        assert_eq!(runtime.write_cursor, 2);
        assert_ne!(runtime.buffer[2], 0xFF);
    }
}
