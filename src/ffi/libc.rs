//! Subset of C standard library.
//!
//! This module implements some functions from libc. Thus it eases linking Drone
//! applications with C libraries.
//!
//! Dynamic memory functions (e.g. `malloc`, `free`) are implemented in terms of
//! [Drone Heap](mod@crate::heap).

use crate::ffi::{c_char, c_int};
use ::alloc::alloc;
use core::{alloc::Layout, ffi::c_void, ptr};

/// A type able to represent the size of any object in bytes.
#[allow(non_camel_case_types)]
pub type size_t = usize;

/// Calculates the length of the string `s`, excluding the terminating null byte
/// (`'\0'`).
///
/// # Safety
///
/// This function works with raw pointers.
#[cfg_attr(not(feature = "std"), no_mangle)]
pub unsafe extern "C" fn strlen(s: *const c_char) -> size_t {
    let mut cursor = s;
    unsafe {
        while *cursor != 0 {
            cursor = cursor.add(1);
        }
    }
    (cursor as size_t) - (s as size_t)
}

/// Returns a pointer to the first occurrence of the character `c` in the string
/// `s`.
///
/// # Safety
///
/// This function works with raw pointers.
#[cfg_attr(not(feature = "std"), no_mangle)]
pub unsafe extern "C" fn strchr(mut s: *const c_char, c: c_int) -> *mut c_char {
    loop {
        unsafe {
            match *s {
                x if x == c as c_char => return s as *mut _,
                0 => return ptr::null_mut(),
                _ => s = s.add(1),
            }
        }
    }
}

/// Compares the two strings `s1` and `s2`. It returns an integer less than,
/// equal to, or greater than zero if `s1` is found, respectively, to be less
/// than, to match, or be greater than `s2`.
///
/// # Safety
///
/// This function works with raw pointers.
#[cfg_attr(not(feature = "std"), no_mangle)]
pub unsafe extern "C" fn strcmp(mut s1: *const c_char, mut s2: *const c_char) -> c_int {
    unsafe {
        while *s1 != 0 && *s1 == *s2 {
            s1 = s1.add(1);
            s2 = s2.add(1);
        }
        c_int::from(*s1) - c_int::from(*s2)
    }
}

/// Allocates size bytes and returns a pointer to the allocated memory. *The
/// memory is not initialized*. If `size` is `0`, then it returns either `NULL`,
/// or a unique pointer value that can later be successfully passed to
/// [`free`](free).
///
/// # Safety
///
/// This function works with raw pointers.
#[cfg_attr(not(feature = "std"), no_mangle)]
pub unsafe extern "C" fn malloc(size: size_t) -> *mut c_void {
    unsafe { alloc::alloc(Layout::from_size_align_unchecked(size, 1)).cast::<c_void>() }
}

/// Allocates memory for an array of `nmemb` elements of `size` bytes each and
/// returns a pointer to the allocated memory. The memory is set to zero. If
/// `nmemb` or `size` is 0, then it returns either `NULL`, or a unique pointer
/// value that can later be successfully passed to [`free`](free).
///
/// # Safety
///
/// This function works with raw pointers.
#[cfg_attr(not(feature = "std"), no_mangle)]
pub unsafe extern "C" fn calloc(nmemb: size_t, size: size_t) -> *mut c_void {
    unsafe {
        alloc::alloc_zeroed(Layout::from_size_align_unchecked(nmemb * size, 1)).cast::<c_void>()
    }
}

/// Changes the size of the memory block pointed to by `ptr` to `size` bytes.
/// The contents will be unchanged in the range from the start of the region up
/// to the minimum of the old and new sizes. If the new size is larger than the
/// old size, the added memory will not be initialized. If `ptr` is `NULL`, then
/// the call is equivalent to `malloc(size)`, for all values of `size`; if
/// `size` is equal to zero, and `ptr` is not `NULL`, then the call is
/// equivalent to `free(ptr)`. Unless `ptr` is `NULL`, it must have been
/// returned by an earlier call to [`malloc`](malloc), [`calloc`](calloc), or
/// [`realloc`](realloc). If the area pointed to was moved, a `free(ptr)` is
/// done.
///
/// # Safety
///
/// This function works with raw pointers.
#[cfg_attr(not(feature = "std"), no_mangle)]
pub unsafe extern "C" fn realloc(ptr: *mut c_void, size: size_t) -> *mut c_void {
    unsafe {
        alloc::realloc(ptr.cast::<u8>(), Layout::from_size_align_unchecked(1, 1), size)
            .cast::<c_void>()
    }
}

/// Frees the memory space pointed to by `ptr`, which must have been returned by
/// a previous call to [`malloc`](malloc), [`calloc`](calloc), or
/// [`realloc`](realloc). Otherwise, or if `free(ptr)` has already been called
/// before, undefined behavior occurs. If `ptr` is `NULL`, no operation is
/// performed.
///
/// # Safety
///
/// This function works with raw pointers.
#[cfg_attr(not(feature = "std"), no_mangle)]
pub unsafe extern "C" fn free(ptr: *mut c_void) {
    unsafe { alloc::dealloc(ptr.cast::<u8>(), Layout::from_size_align_unchecked(1, 1)) }
}
