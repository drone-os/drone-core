// Ported from rustc b99172311

//! Utilities related to FFI bindings.
//!
//! This module provides utilities to handle data across non-Rust interfaces,
//! like other programming languages and the underlying operating system. It is
//! mainly of use for FFI (Foreign Function Interface) bindings and code that
//! needs to exchange C-like strings with other languages.
//!
//! # Overview
//!
//! Rust represents owned strings with the `String` type, and borrowed slices of
//! strings with the `str` primitive. Both are always in UTF-8 encoding, and may
//! contain nul bytes in the middle, i.e. if you look at the bytes that make up
//! the string, there may be a `\0` among them. Both `String` and `str` store
//! their length explicitly; there are no nul terminators at the end of strings
//! like in C.
//!
//! C strings are different from Rust strings:
//!
//! * **Encodings** - Rust strings are UTF-8, but C strings may use other
//! encodings. If you are using a string from C, you should check its encoding
//! explicitly, rather than just assuming that it is UTF-8 like you can do in
//! Rust.
//!
//! * **Character size** - C strings may use `char` or `wchar_t`-sized
//! characters; please **note** that C's `char` is different from Rust's.  The C
//! standard leaves the actual sizes of those types open to interpretation, but
//! defines different APIs for strings made up of each character type. Rust
//! strings are always UTF-8, so different Unicode characters will be encoded in
//! a variable number of bytes each. The Rust type `char` represents a '[Unicode
//! scalar value]', which is similar to, but not the same as, a '[Unicode code
//! point]'.
//!
//! * **Nul terminators and implicit string lengths** - Often, C strings are
//! nul-terminated, i.e. they have a `\0` character at the end. The length of a
//! string buffer is not stored, but has to be calculated; to compute the length
//! of a string, C code must manually call a function like `strlen()` for
//! `char`-based strings, or `wcslen()` for `wchar_t`-based ones. Those
//! functions return the number of characters in the string excluding the nul
//! terminator, so the buffer length is really `len+1` characters.  Rust strings
//! don't have a nul terminator; their length is always stored and does not need
//! to be calculated. While in Rust accessing a string's length is a O(1)
//! operation (because the length is stored); in C it is an O(length) operation
//! because the length needs to be computed by scanning the string for the nul
//! terminator.
//!
//! * **Internal nul characters** - When C strings have a nul terminator
//! character, this usually means that they cannot have nul characters in the
//! middle — a nul character would essentially truncate the string. Rust strings
//! *can* have nul characters in the middle, because nul does not have to mark
//! the end of the string in Rust.
//!
//! # Representations of non-Rust strings
//!
//! [`CString`] and [`CStr`] are useful when you need to transfer UTF-8 strings
//! to and from languages with a C ABI, like Python.
//!
//! * **From Rust to C:** [`CString`] represents an owned, C-friendly string: it
//! is nul-terminated, and has no internal nul characters.  Rust code can create
//! a `CString` out of a normal string (provided that the string doesn't have
//! nul characters in the middle), and then use a variety of methods to obtain a
//! raw `*mut u8` that can then be passed as an argument to functions which use
//! the C conventions for strings.
//!
//! * **From C to Rust:** [`CStr`] represents a borrowed C string; it is what
//! you would use to wrap a raw `*const u8` that you got from a C function. A
//! `CStr` is guaranteed to be a nul-terminated array of bytes. Once you have a
//! `CStr`, you can convert it to a Rust `&str` if it's valid UTF-8, or lossily
//! convert it by adding replacement characters.
//!
//! [Unicode scalar value]: http://www.unicode.org/glossary/#unicode_scalar_value
//! [Unicode code point]: http://www.unicode.org/glossary/#code_point
//! [`CString`]: CString
//! [`CStr`]: CStr

mod str;
mod string;

pub use self::str::{CStr, FromBytesWithNulError};
pub use self::string::{CString, IntoStringError, NulError};
pub use drone_ctypes::*;

unsafe fn strlen(ptr: *const c_char) -> usize {
  let mut cursor = ptr;
  while *cursor != 0 {
    cursor = cursor.add(1);
  }
  (cursor as usize) - (ptr as usize)
}

#[cfg(test)]
mod tests {
  use super::*;
  use alloc::arc::Arc;
  use alloc::borrow::Cow::{Borrowed, Owned};
  use alloc::rc::Rc;
  use core::hash::{Hash, Hasher};
  use std::collections::hash_map::DefaultHasher;

  #[test]
  fn c_to_rust() {
    let data = b"123\0";
    let ptr = data.as_ptr() as *const c_char;
    unsafe {
      assert_eq!(CStr::from_ptr(ptr).to_bytes(), b"123");
      assert_eq!(
        CStr::from_ptr(ptr).to_bytes_with_nul(),
        b"123\0"
      );
    }
  }

  #[test]
  fn simple() {
    let s = CString::new("1234").unwrap();
    assert_eq!(s.as_bytes(), b"1234");
    assert_eq!(s.as_bytes_with_nul(), b"1234\0");
  }

  #[test]
  fn build_with_zero1() {
    assert!(CString::new(&b"\0"[..]).is_err());
  }
  #[test]
  fn build_with_zero2() {
    assert!(CString::new(vec![0]).is_err());
  }

  #[test]
  fn build_with_zero3() {
    unsafe {
      let s = CString::from_vec_unchecked(vec![0]);
      assert_eq!(s.as_bytes(), b"\0");
    }
  }

  #[test]
  fn formatted() {
    let s = CString::new(&b"abc\x01\x02\n\xE2\x80\xA6\xFF"[..]).unwrap();
    assert_eq!(
      format!("{:?}", s),
      r#""abc\x01\x02\n\xe2\x80\xa6\xff""#
    );
  }

  #[test]
  fn borrowed() {
    unsafe {
      let s = CStr::from_ptr(b"12\0".as_ptr() as *const _);
      assert_eq!(s.to_bytes(), b"12");
      assert_eq!(s.to_bytes_with_nul(), b"12\0");
    }
  }

  #[test]
  fn to_str() {
    let data = b"123\xE2\x80\xA6\0";
    let ptr = data.as_ptr() as *const c_char;
    unsafe {
      assert_eq!(CStr::from_ptr(ptr).to_str(), Ok("123…"));
      assert_eq!(
        CStr::from_ptr(ptr).to_string_lossy(),
        Borrowed("123…")
      );
    }
    let data = b"123\xE2\0";
    let ptr = data.as_ptr() as *const c_char;
    unsafe {
      assert!(CStr::from_ptr(ptr).to_str().is_err());
      assert_eq!(
        CStr::from_ptr(ptr).to_string_lossy(),
        Owned::<str>(format!("123\u{FFFD}"))
      );
    }
  }

  #[test]
  fn to_owned() {
    let data = b"123\0";
    let ptr = data.as_ptr() as *const c_char;

    let owned = unsafe { CStr::from_ptr(ptr).to_owned() };
    assert_eq!(owned.as_bytes_with_nul(), data);
  }

  #[test]
  fn equal_hash() {
    let data = b"123\xE2\xFA\xA6\0";
    let ptr = data.as_ptr() as *const c_char;
    let cstr: &'static CStr = unsafe { CStr::from_ptr(ptr) };

    let mut s = DefaultHasher::new();
    cstr.hash(&mut s);
    let cstr_hash = s.finish();
    let mut s = DefaultHasher::new();
    CString::new(&data[..data.len() - 1])
      .unwrap()
      .hash(&mut s);
    let cstring_hash = s.finish();

    assert_eq!(cstr_hash, cstring_hash);
  }

  #[test]
  fn from_bytes_with_nul() {
    let data = b"123\0";
    let cstr = CStr::from_bytes_with_nul(data);
    assert_eq!(cstr.map(CStr::to_bytes), Ok(&b"123"[..]));
    let cstr = CStr::from_bytes_with_nul(data);
    assert_eq!(
      cstr.map(CStr::to_bytes_with_nul),
      Ok(&b"123\0"[..])
    );

    unsafe {
      let cstr = CStr::from_bytes_with_nul(data);
      let cstr_unchecked = CStr::from_bytes_with_nul_unchecked(data);
      assert_eq!(cstr, Ok(cstr_unchecked));
    }
  }

  #[test]
  fn from_bytes_with_nul_unterminated() {
    let data = b"123";
    let cstr = CStr::from_bytes_with_nul(data);
    assert!(cstr.is_err());
  }

  #[test]
  fn from_bytes_with_nul_interior() {
    let data = b"1\023\0";
    let cstr = CStr::from_bytes_with_nul(data);
    assert!(cstr.is_err());
  }

  #[test]
  fn into_boxed() {
    let orig: &[u8] = b"Hello, world!\0";
    let cstr = CStr::from_bytes_with_nul(orig).unwrap();
    let boxed: Box<CStr> = Box::from(cstr);
    let cstring = cstr
      .to_owned()
      .into_boxed_c_str()
      .into_c_string();
    assert_eq!(cstr, &*boxed);
    assert_eq!(&*boxed, &*cstring);
    assert_eq!(&*cstring, cstr);
  }

  #[test]
  fn boxed_default() {
    let boxed = <Box<CStr>>::default();
    assert_eq!(boxed.to_bytes_with_nul(), &[0]);
  }

  #[test]
  fn into_rc() {
    let orig: &[u8] = b"Hello, world!\0";
    let cstr = CStr::from_bytes_with_nul(orig).unwrap();
    let rc: Rc<CStr> = Rc::from(cstr);
    let arc: Arc<CStr> = Arc::from(cstr);

    assert_eq!(&*rc, cstr);
    assert_eq!(&*arc, cstr);

    let rc2: Rc<CStr> = Rc::from(cstr.to_owned());
    let arc2: Arc<CStr> = Arc::from(cstr.to_owned());

    assert_eq!(&*rc2, cstr);
    assert_eq!(&*arc2, cstr);
  }
}
