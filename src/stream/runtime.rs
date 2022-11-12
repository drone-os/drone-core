#![cfg_attr(feature = "std", allow(unused_imports, unused_variables))]

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

    #[allow(clippy::blocks_in_if_conditions)]
    #[inline(never)]
    #[export_name = "stream_write_transaction"]
    unsafe fn write_transaction(&mut self, stream: u8, buffer: *const u8, length: u8) {
        #[cfg(feature = "std")]
        return unimplemented!();
        #[cfg(not(feature = "std"))]
        unsafe {
            while Interrupts::paused(|| {
                let read_cursor = ptr::addr_of!(self.read_cursor).read_volatile();
                let write_cursor = ptr::addr_of!(self.write_cursor).read_volatile();
                let wrapped = write_cursor >= read_cursor;
                let available = if wrapped { self.buffer_size } else { read_cursor } - write_cursor;
                let frame_length = u32::from(length) + HEADER_LENGTH;
                let cursor =
                    ptr::addr_of_mut!(*self).add(1).cast::<u8>().add(write_cursor as usize);
                if available >= frame_length {
                    let mut next_write_cursor = write_cursor + frame_length;
                    if next_write_cursor == self.buffer_size {
                        next_write_cursor = 0;
                    }
                    if next_write_cursor == read_cursor {
                        return true;
                    }
                    *cursor = stream;
                    *cursor.add(1) = length;
                    cursor.add(2).copy_from_nonoverlapping(buffer, usize::from(length));
                    ptr::addr_of_mut!(self.write_cursor).write_volatile(next_write_cursor);
                    return false;
                }
                if wrapped {
                    if available > HEADER_LENGTH {
                        *cursor = 0xFF;
                        *cursor.add(1) = (available - HEADER_LENGTH) as u8;
                    }
                    ptr::addr_of_mut!(self.write_cursor).write_volatile(0);
                }
                true
            }) {}
        }
    }
}
