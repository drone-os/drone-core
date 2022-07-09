use core::{
    cell::UnsafeCell,
    cmp::min,
    ptr,
    sync::atomic::{AtomicU8, AtomicUsize, Ordering},
};

extern "C" {
    static STREAM_START: UnsafeCell<u8>;
    static STREAM_END: UnsafeCell<u8>;
}

#[repr(C)]
pub struct Runtime {
    mask: AtomicU8,
    write_offset: AtomicUsize,
    read_offset: AtomicUsize,
}

impl Runtime {
    pub const fn new() -> Self {
        Self {
            mask: AtomicU8::new(0),
            write_offset: AtomicUsize::new(0),
            read_offset: AtomicUsize::new(0),
        }
    }

    pub fn is_enabled(&self, stream: u8) -> bool {
        self.mask.load(Ordering::Relaxed) & 1 << stream != 0
    }

    pub fn write_bytes(&self, _stream: u8, mut buffer: *const u8, mut length: usize) {
        let buffer_size = unsafe { STREAM_END.get() as usize - STREAM_START.get() as usize };
        let mut write_offset = self.write_offset.load(Ordering::Relaxed);
        loop {
            let read_offset = self.read_offset.load(Ordering::Relaxed);
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
            self.write_offset.store(write_offset, Ordering::Relaxed);
            if length == 0 {
                break;
            }
        }
    }

    #[allow(unused_variables, clippy::unused_self)]
    pub fn write_u8(&self, stream: u8, value: u8) {
        todo!()
    }

    #[allow(unused_variables, clippy::unused_self)]
    pub fn write_u16(&self, stream: u8, value: u16) {
        todo!()
    }

    #[allow(unused_variables, clippy::unused_self)]
    pub fn write_u32(&self, stream: u8, value: u32) {
        todo!()
    }
}
