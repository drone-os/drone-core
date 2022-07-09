use super::{STREAM_END, STREAM_START};
use core::{
    cmp::min,
    ptr,
    sync::atomic::{AtomicU32, Ordering},
};
use drone_stream::Runtime;

pub trait LocalRuntime {
    fn is_enabled(&self, stream: u8) -> bool;

    fn write_bytes(&self, stream: u8, buffer: *const u8, length: usize);

    fn write_u8(&self, stream: u8, value: u8);

    fn write_u16(&self, stream: u8, value: u16);

    fn write_u32(&self, stream: u8, value: u32);
}

trait AtomicRuntime {
    fn atomic_mask(&self) -> &AtomicU32;

    fn atomic_read_offset(&self) -> &AtomicU32;

    fn atomic_write_offset(&self) -> &AtomicU32;
}

impl LocalRuntime for Runtime {
    fn is_enabled(&self, stream: u8) -> bool {
        self.atomic_mask().load(Ordering::Relaxed) & 1 << stream != 0
    }

    fn write_bytes(&self, _stream: u8, mut buffer: *const u8, mut length: usize) {
        let buffer_size = unsafe { STREAM_END.get() as usize - STREAM_START.get() as usize };
        let mut write_offset = self.atomic_write_offset().load(Ordering::Relaxed) as usize;
        loop {
            let read_offset = self.atomic_read_offset().load(Ordering::Relaxed) as usize;
            let mut bytes_to_write = read_offset.wrapping_sub(write_offset).wrapping_sub(1);
            if write_offset > read_offset {
                bytes_to_write = bytes_to_write.wrapping_add(buffer_size);
            }
            bytes_to_write = min(bytes_to_write, buffer_size - write_offset);
            bytes_to_write = min(bytes_to_write, length);
            let dst = unsafe { STREAM_START.get().add(write_offset) };
            unsafe { ptr::copy_nonoverlapping(buffer, dst, bytes_to_write) };
            buffer = unsafe { buffer.add(bytes_to_write) };
            length -= bytes_to_write;
            write_offset += bytes_to_write;
            if write_offset == buffer_size {
                write_offset = 0;
            }
            self.atomic_write_offset().store(write_offset as u32, Ordering::Relaxed);
            if length == 0 {
                break;
            }
        }
    }

    #[allow(unused_variables, clippy::unused_self)]
    fn write_u8(&self, stream: u8, value: u8) {
        todo!()
    }

    #[allow(unused_variables, clippy::unused_self)]
    fn write_u16(&self, stream: u8, value: u16) {
        todo!()
    }

    #[allow(unused_variables, clippy::unused_self)]
    fn write_u32(&self, stream: u8, value: u32) {
        todo!()
    }
}

impl AtomicRuntime for Runtime {
    fn atomic_mask(&self) -> &AtomicU32 {
        unsafe { &*ptr::addr_of!(self.mask).cast() }
    }

    fn atomic_read_offset(&self) -> &AtomicU32 {
        unsafe { &*ptr::addr_of!(self.read_offset).cast() }
    }

    fn atomic_write_offset(&self) -> &AtomicU32 {
        unsafe { &*ptr::addr_of!(self.write_offset).cast() }
    }
}
