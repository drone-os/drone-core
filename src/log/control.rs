use core::{
    cell::UnsafeCell,
    cmp::min,
    ptr,
    sync::atomic::{AtomicU32, AtomicUsize, Ordering},
};

extern "C" {
    static LOG_START: UnsafeCell<u8>;
    static LOG_END: UnsafeCell<u8>;
}

#[repr(C)]
pub(crate) struct Control {
    state: AtomicU32,
    write_offset: AtomicUsize,
    read_offset: AtomicUsize,
}

impl Control {
    pub(crate) const fn new() -> Self {
        Self {
            state: AtomicU32::new(0),
            write_offset: AtomicUsize::new(0),
            read_offset: AtomicUsize::new(0),
        }
    }

    pub(crate) fn is_enabled(&self, stream: u8) -> bool {
        self.state.load(Ordering::Relaxed) & 1 << stream != 0
    }

    pub(crate) fn write_bytes(&self, _stream: u8, mut buffer: *const u8, mut length: usize) {
        let mut write_offset = self.write_offset.load(Ordering::Relaxed);
        loop {
            let read_offset = self.read_offset.load(Ordering::Relaxed);
            let mut bytes_to_write = read_offset.wrapping_sub(write_offset).wrapping_sub(1);
            if write_offset > read_offset {
                bytes_to_write = bytes_to_write.wrapping_add(log_size());
            }
            bytes_to_write = min(bytes_to_write, log_size() - write_offset);
            bytes_to_write = min(bytes_to_write, length);
            let dst = unsafe { LOG_START.get().add(write_offset) };
            unsafe { ptr::copy_nonoverlapping(buffer, dst, bytes_to_write) };
            buffer = unsafe { buffer.add(bytes_to_write) };
            length -= bytes_to_write;
            write_offset += bytes_to_write;
            if write_offset == log_size() {
                write_offset = 0;
            }
            self.write_offset.store(write_offset, Ordering::Relaxed);
            if length == 0 {
                break;
            }
        }
    }

    #[allow(unused_variables, clippy::unused_self)]
    pub(crate) fn write_u8(&self, stream: u8, value: u8) {
        todo!()
    }

    #[allow(unused_variables, clippy::unused_self)]
    pub(crate) fn write_u16(&self, stream: u8, value: u16) {
        todo!()
    }

    #[allow(unused_variables, clippy::unused_self)]
    pub(crate) fn write_u32(&self, stream: u8, value: u32) {
        todo!()
    }
}

fn log_size() -> usize {
    unsafe { LOG_END.get() as usize - LOG_START.get() as usize }
}
