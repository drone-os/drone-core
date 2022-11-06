use crate::stream::Stream;
use core::alloc::Layout;
use core::mem;

#[inline(always)]
pub fn allocate(trace_stream: u8, layout: Layout) {
    #[inline(never)]
    fn trace(trace_stream: u8, layout: Layout) {
        let buffer: [usize; 2] = [0_usize.to_be(), layout.size()];
        let buffer: [u8; mem::size_of::<[usize; 2]>()] = unsafe { mem::transmute(buffer) };
        Stream::new(trace_stream).write_transaction(&buffer[3..]);
    }
    if Stream::new(trace_stream).is_enabled() {
        trace(trace_stream, layout);
    }
}

#[inline(always)]
pub fn deallocate(trace_stream: u8, layout: Layout) {
    #[inline(never)]
    fn trace(trace_stream: u8, layout: Layout) {
        let buffer: [usize; 2] = [1_usize.to_be(), layout.size()];
        let buffer: [u8; mem::size_of::<[usize; 2]>()] = unsafe { mem::transmute(buffer) };
        Stream::new(trace_stream).write_transaction(&buffer[3..]);
    }
    if Stream::new(trace_stream).is_enabled() {
        trace(trace_stream, layout);
    }
}

#[inline(always)]
pub fn grow(trace_stream: u8, old_layout: Layout, new_layout: Layout) {
    #[inline(never)]
    fn trace(trace_stream: u8, old_layout: Layout, new_layout: Layout) {
        let buffer: [usize; 3] = [2_usize.to_be(), old_layout.size(), new_layout.size()];
        let buffer: [u8; mem::size_of::<[usize; 3]>()] = unsafe { mem::transmute(buffer) };
        Stream::new(trace_stream).write_transaction(&buffer[3..]);
    }
    if Stream::new(trace_stream).is_enabled() {
        trace(trace_stream, old_layout, new_layout);
    }
}

#[inline(always)]
pub fn shrink(trace_stream: u8, old_layout: Layout, new_layout: Layout) {
    #[inline(never)]
    fn trace(trace_stream: u8, old_layout: Layout, new_layout: Layout) {
        let buffer: [usize; 3] = [3_usize.to_be(), old_layout.size(), new_layout.size()];
        let buffer: [u8; mem::size_of::<[usize; 3]>()] = unsafe { mem::transmute(buffer) };
        Stream::new(trace_stream).write_transaction(&buffer[3..]);
    }
    if Stream::new(trace_stream).is_enabled() {
        trace(trace_stream, old_layout, new_layout);
    }
}
