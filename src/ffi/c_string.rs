use crate::ffi::{c_char, libc::strlen, CStr};
use alloc::borrow::{Borrow, Cow};
use core::{
    fmt, mem, ops, ptr,
    slice::{self, memchr},
    str::Utf8Error,
};

/// A type representing an owned, C-compatible, nul-terminated string with no
/// nul bytes in the middle.
///
/// This type serves the purpose of being able to safely generate a C-compatible
/// string from a Rust byte slice or vector. An instance of this type is a
/// static guarantee that the underlying bytes contain no interior 0 bytes ("nul
/// characters") and that the final byte is 0 ("nul terminator").
///
/// `CString` is to `&`[`CStr`] as [`String`] is to `&`[`str`]: the former in
/// each pair are owned strings; the latter are borrowed references.
///
/// # Creating a `CString`
///
/// A `CString` is created from either a byte slice or a byte vector, or
/// anything that implements [`Into`]`<`[`Vec`]`<`[`u8`]`>>` (for example, you
/// can build a `CString` straight out of a [`String`] or a `&`[`str`], since
/// both implement that trait).
///
/// The [`CString::new`] method will actually check that the provided `&[u8]`
/// does not have 0 bytes in the middle, and return an error if it finds one.
///
/// # Extracting a raw pointer to the whole C string
///
/// `CString` implements a `as_ptr` method through the [`core::ops::Deref`]
/// trait. This method will give you a `*const c_char` which you can feed
/// directly to extern functions that expect a nul-terminated string, like C's
/// `strdup()`. Notice that `as_ptr` returns a read-only pointer; if the C code
/// writes to it, that causes undefined behavior.
///
/// # Extracting a slice of the whole C string
///
/// Alternatively, you can obtain a `&[`[`u8`]`]` slice from a `CString` with
/// the [`CString::as_bytes`] method. Slices produced in this way do *not*
/// contain the trailing nul terminator. This is useful when you will be calling
/// an extern function that takes a `*const u8` argument which is not
/// necessarily nul-terminated, plus another argument with the length of the
/// string — like C's `strndup()`. You can of course get the slice's length with
/// its `len` method.
///
/// If you need a `&[`[`u8`]`]` slice *with* the nul terminator, you can use
/// [`CString::as_bytes_with_nul`] instead.
///
/// Once you have the kind of slice you need (with or without a nul terminator),
/// you can call the slice's own `as_ptr` method to get a read-only raw pointer
/// to pass to extern functions. See the documentation for that function for a
/// discussion on ensuring the lifetime of the raw pointer.
///
/// # Examples
///
/// ```
/// # fn main() {
/// use drone_core::ffi::{c_char, CString};
///
/// extern "C" fn my_printer(s: *const c_char) {}
///
/// // We are certain that our string doesn't have 0 bytes in the middle,
/// // so we can .expect()
/// let c_to_print = CString::new("Hello, world!").expect("CString::new failed");
/// unsafe {
///     my_printer(c_to_print.as_ptr());
/// }
/// # }
/// ```
///
/// # Safety
///
/// `CString` is intended for working with traditional C-style strings (a
/// sequence of non-nul bytes terminated by a single nul byte); the primary use
/// case for these kinds of strings is interoperating with C-like code. Often
/// you will need to transfer ownership to/from that external code. It is
/// strongly recommended that you thoroughly read through the documentation of
/// `CString` before use, as improper ownership management of `CString`
/// instances can lead to invalid memory accesses, memory leaks, and other
/// memory errors.
#[derive(PartialEq, PartialOrd, Eq, Ord, Hash, Clone)]
pub struct CString {
    // Invariant 1: the slice ends with a zero byte and has a length of at least one.
    // Invariant 2: the slice contains only one zero byte.
    // Improper usage of unsafe function can break Invariant 2, but not Invariant 1.
    pub(super) inner: Box<[u8]>,
}

/// An error indicating that an interior nul byte was found.
///
/// While Rust strings may contain nul bytes in the middle, C strings can't, as
/// that byte would effectively truncate the string.
///
/// This error is created by the [`new`](CString::new) method on [`CString`].
/// See its documentation for more.
///
/// # Examples
///
/// ```
/// use drone_core::ffi::{CString, NulError};
///
/// let _: NulError = CString::new(b"f\0oo".to_vec()).unwrap_err();
/// ```
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct NulError(usize, Vec<u8>);

/// An error indicating invalid UTF-8 when converting a [`CString`] into a
/// [`String`].
///
/// `CString` is just a wrapper over a buffer of bytes with a nul terminator;
/// [`CString::into_string`] performs UTF-8 validation on those bytes and may
/// return this error.
///
/// This `struct` is created by the [`CString::into_string`] method on
/// [`CString`]. See its documentation for more.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct IntoStringError {
    inner: CString,
    error: Utf8Error,
}

impl CString {
    /// Creates a new C-compatible string from a container of bytes.
    ///
    /// This function will consume the provided data and use the underlying
    /// bytes to construct a new string, ensuring that there is a trailing 0
    /// byte. This trailing 0 byte will be appended by this function; the
    /// provided data should *not* contain any 0 bytes in it.
    ///
    /// # Examples
    ///
    /// ```
    /// use drone_core::ffi::{c_char, CString};
    ///
    /// extern "C" fn puts(_s: *const c_char) {}
    ///
    /// let to_print = CString::new("Hello!").expect("CString::new failed");
    /// unsafe {
    ///     puts(to_print.as_ptr());
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// This function will return an error if the supplied bytes contain an
    /// internal 0 byte. The [`NulError`] returned will contain the bytes as
    /// well as the position of the nul byte.
    pub fn new<T: Into<Vec<u8>>>(t: T) -> Result<Self, NulError> {
        Self::_new(t.into())
    }

    fn _new(bytes: Vec<u8>) -> Result<Self, NulError> {
        match memchr::memchr(0, &bytes) {
            Some(i) => Err(NulError(i, bytes)),
            None => Ok(unsafe { Self::from_vec_unchecked(bytes) }),
        }
    }

    /// Creates a C-compatible string by consuming a byte vector, without
    /// checking for interior 0 bytes.
    ///
    /// This method is equivalent to [`CString::new`] except that no runtime
    /// assertion is made that `v` contains no 0 bytes, and it requires an
    /// actual byte vector, not anything that can be converted to one with
    /// [`Into`].
    ///
    /// # Examples
    ///
    /// ```
    /// use drone_core::ffi::CString;
    ///
    /// let raw = b"foo".to_vec();
    /// unsafe {
    ///     let c_string = CString::from_vec_unchecked(raw);
    /// }
    /// ```
    pub unsafe fn from_vec_unchecked(mut v: Vec<u8>) -> Self {
        v.reserve_exact(1);
        v.push(0);
        Self {
            inner: v.into_boxed_slice(),
        }
    }

    /// Retakes ownership of a `CString` that was transferred to C via
    /// [`CString::into_raw`].
    ///
    /// Additionally, the length of the string will be recalculated from the
    /// pointer.
    ///
    /// # Safety
    ///
    /// This should only ever be called with a pointer that was earlier obtained
    /// by calling [`CString::into_raw`] on a `CString`. Other usage (e.g.,
    /// trying to take ownership of a string that was allocated by foreign
    /// code) is likely to lead to undefined behavior or allocator
    /// corruption.
    ///
    /// > **Note:** If you need to borrow a string that was allocated by
    /// > foreign code, use [`CStr`]. If you need to take ownership of
    /// > a string that was allocated by foreign code, you will need to
    /// > make your own provisions for freeing it appropriately, likely
    /// > with the foreign code's API to do that.
    ///
    /// # Examples
    ///
    /// Creates a `CString`, pass ownership to an `extern` function (via raw
    /// pointer), then retake ownership with `from_raw`:
    ///
    /// ```
    /// use drone_core::ffi::{c_char, CString};
    ///
    /// extern "C" fn some_extern_function(_s: *mut c_char) {}
    ///
    /// let c_string = CString::new("Hello!").expect("CString::new failed");
    /// let raw = c_string.into_raw();
    /// unsafe {
    ///     some_extern_function(raw);
    ///     let c_string = CString::from_raw(raw);
    /// }
    /// ```
    pub unsafe fn from_raw(ptr: *mut c_char) -> Self {
        let len = strlen(ptr) + 1; // Including the NUL byte
        let slice = slice::from_raw_parts_mut(ptr, len as usize);
        Self {
            inner: Box::from_raw(slice as *mut [c_char] as *mut [u8]),
        }
    }

    /// Consumes the `CString` and transfers ownership of the string to a C
    /// caller.
    ///
    /// The pointer which this function returns must be returned to Rust and
    /// reconstituted using [`CString::from_raw`] to be properly deallocated.
    /// Specifically, one should *not* use the standard C `free()` function
    /// to deallocate this string.
    ///
    /// Failure to call [`CString::from_raw`] will lead to a memory leak.
    ///
    /// # Examples
    ///
    /// ```
    /// use drone_core::ffi::CString;
    ///
    /// let c_string = CString::new("foo").expect("CString::new failed");
    ///
    /// let ptr = c_string.into_raw();
    ///
    /// unsafe {
    ///     assert_eq!(b'f', *ptr as u8);
    ///     assert_eq!(b'o', *ptr.offset(1) as u8);
    ///     assert_eq!(b'o', *ptr.offset(2) as u8);
    ///     assert_eq!(b'\0', *ptr.offset(3) as u8);
    ///
    ///     // retake pointer to free memory
    ///     let _ = CString::from_raw(ptr);
    /// }
    /// ```
    #[inline]
    pub fn into_raw(self) -> *mut c_char {
        Box::into_raw(self.into_inner()) as *mut c_char
    }

    /// Converts the `CString` into a [`String`] if it contains valid UTF-8
    /// data.
    ///
    /// On failure, ownership of the original `CString` is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use drone_core::ffi::CString;
    ///
    /// let valid_utf8 = vec![b'f', b'o', b'o'];
    /// let cstring = CString::new(valid_utf8).expect("CString::new failed");
    /// assert_eq!(
    ///     cstring.into_string().expect("into_string() call failed"),
    ///     "foo"
    /// );
    ///
    /// let invalid_utf8 = vec![b'f', 0xff, b'o', b'o'];
    /// let cstring = CString::new(invalid_utf8).expect("CString::new failed");
    /// let err = cstring
    ///     .into_string()
    ///     .err()
    ///     .expect("into_string().err() failed");
    /// assert_eq!(err.utf8_error().valid_up_to(), 1);
    /// ```
    pub fn into_string(self) -> Result<String, IntoStringError> {
        String::from_utf8(self.into_bytes()).map_err(|e| IntoStringError {
            error: e.utf8_error(),
            inner: unsafe { Self::from_vec_unchecked(e.into_bytes()) },
        })
    }

    /// Consumes the `CString` and returns the underlying byte buffer.
    ///
    /// The returned buffer does **not** contain the trailing nul terminator,
    /// and it is guaranteed to not have any interior nul bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use drone_core::ffi::CString;
    ///
    /// let c_string = CString::new("foo").expect("CString::new failed");
    /// let bytes = c_string.into_bytes();
    /// assert_eq!(bytes, vec![b'f', b'o', b'o']);
    /// ```
    pub fn into_bytes(self) -> Vec<u8> {
        let mut vec = self.into_inner().into_vec();
        let nul = vec.pop();
        debug_assert_eq!(nul, Some(0_u8));
        vec
    }

    /// Equivalent to the [`CString::into_bytes`] function except that the
    /// returned vector includes the trailing nul terminator.
    ///
    /// # Examples
    ///
    /// ```
    /// use drone_core::ffi::CString;
    ///
    /// let c_string = CString::new("foo").expect("CString::new failed");
    /// let bytes = c_string.into_bytes_with_nul();
    /// assert_eq!(bytes, vec![b'f', b'o', b'o', b'\0']);
    /// ```
    pub fn into_bytes_with_nul(self) -> Vec<u8> {
        self.into_inner().into_vec()
    }

    /// Returns the contents of this `CString` as a slice of bytes.
    ///
    /// The returned slice does **not** contain the trailing nul terminator, and
    /// it is guaranteed to not have any interior nul bytes. If you need the nul
    /// terminator, use [`CString::as_bytes_with_nul`] instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use drone_core::ffi::CString;
    ///
    /// let c_string = CString::new("foo").expect("CString::new failed");
    /// let bytes = c_string.as_bytes();
    /// assert_eq!(bytes, &[b'f', b'o', b'o']);
    /// ```
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.inner[..self.inner.len() - 1]
    }

    /// Equivalent to the [`CString::as_bytes`] function except that the
    /// returned slice includes the trailing nul terminator.
    ///
    /// # Examples
    ///
    /// ```
    /// use drone_core::ffi::CString;
    ///
    /// let c_string = CString::new("foo").expect("CString::new failed");
    /// let bytes = c_string.as_bytes_with_nul();
    /// assert_eq!(bytes, &[b'f', b'o', b'o', b'\0']);
    /// ```
    #[inline]
    pub fn as_bytes_with_nul(&self) -> &[u8] {
        &self.inner
    }

    /// Extracts a [`CStr`] slice containing the entire string.
    ///
    /// # Examples
    ///
    /// ```
    /// use drone_core::ffi::{CStr, CString};
    ///
    /// let c_string = CString::new(b"foo".to_vec()).expect("CString::new failed");
    /// let c_str = c_string.as_c_str();
    /// assert_eq!(
    ///     c_str,
    ///     CStr::from_bytes_with_nul(b"foo\0").expect("CStr::from_bytes_with_nul failed")
    /// );
    /// ```
    #[inline]
    pub fn as_c_str(&self) -> &CStr {
        &*self
    }

    /// Converts this `CString` into a boxed [`CStr`].
    ///
    /// # Examples
    ///
    /// ```
    /// use drone_core::ffi::{CStr, CString};
    ///
    /// let c_string = CString::new(b"foo".to_vec()).expect("CString::new failed");
    /// let boxed = c_string.into_boxed_c_str();
    /// assert_eq!(
    ///     &*boxed,
    ///     CStr::from_bytes_with_nul(b"foo\0").expect("CStr::from_bytes_with_nul failed")
    /// );
    /// ```
    pub fn into_boxed_c_str(self) -> Box<CStr> {
        unsafe { Box::from_raw(Box::into_raw(self.into_inner()) as *mut CStr) }
    }

    /// Bypass "move out of struct which implements [`Drop`] trait" restriction.
    pub(super) fn into_inner(self) -> Box<[u8]> {
        // Rationale: `mem::forget(self)` invalidates the previous call to
        // `ptr::read(&self.inner)` so we use `ManuallyDrop` to ensure `self` is
        // not dropped. Then we can return the box directly without invalidating
        // it. See https://github.com/rust-lang/rust/issues/62553.
        let this = mem::ManuallyDrop::new(self);
        unsafe { ptr::read(&this.inner) }
    }
}

impl NulError {
    /// Returns the position of the nul byte in the slice that caused
    /// [`CString::new`] to fail.
    ///
    /// # Examples
    ///
    /// ```
    /// use drone_core::ffi::CString;
    ///
    /// let nul_error = CString::new("foo\0bar").unwrap_err();
    /// assert_eq!(nul_error.nul_position(), 3);
    ///
    /// let nul_error = CString::new("foo bar\0").unwrap_err();
    /// assert_eq!(nul_error.nul_position(), 7);
    /// ```
    pub fn nul_position(&self) -> usize {
        self.0
    }

    /// Consumes this error, returning the underlying vector of bytes which
    /// generated the error in the first place.
    ///
    /// # Examples
    ///
    /// ```
    /// use drone_core::ffi::CString;
    ///
    /// let nul_error = CString::new("foo\0bar").unwrap_err();
    /// assert_eq!(nul_error.into_vec(), b"foo\0bar");
    /// ```
    pub fn into_vec(self) -> Vec<u8> {
        self.1
    }
}

impl IntoStringError {
    /// Consumes this error, returning original [`CString`] which generated the
    /// error.
    pub fn into_cstring(self) -> CString {
        self.inner
    }

    /// Access the underlying UTF-8 error that was the cause of this error.
    pub fn utf8_error(&self) -> Utf8Error {
        self.error
    }
}

// Turns this `CString` into an empty string to prevent memory unsafe code from
// working by accident. Inline to prevent LLVM from optimizing it away in debug
// builds.
impl Drop for CString {
    #[inline]
    fn drop(&mut self) {
        unsafe { *self.inner.get_unchecked_mut(0) = 0 };
    }
}

impl ops::Deref for CString {
    type Target = CStr;

    #[inline]
    fn deref(&self) -> &CStr {
        unsafe { CStr::from_bytes_with_nul_unchecked(self.as_bytes_with_nul()) }
    }
}

impl fmt::Debug for CString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl From<CString> for Vec<u8> {
    /// Converts a [`CString`] into a [`Vec`]`<u8>`.
    ///
    /// The conversion consumes the [`CString`], and removes the terminating NUL
    /// byte.
    #[inline]
    fn from(s: CString) -> Self {
        s.into_bytes()
    }
}

impl Default for CString {
    /// Creates an empty `CString`.
    fn default() -> Self {
        let a: &CStr = Default::default();
        a.to_owned()
    }
}

impl Borrow<CStr> for CString {
    #[inline]
    fn borrow(&self) -> &CStr {
        self
    }
}

impl From<&CStr> for CString {
    fn from(s: &CStr) -> Self {
        s.to_owned()
    }
}

impl<'a> From<Cow<'a, CStr>> for CString {
    #[inline]
    fn from(s: Cow<'a, CStr>) -> Self {
        s.into_owned()
    }
}

impl From<Box<CStr>> for CString {
    /// Converts a [`Box`]`<CStr>` into a [`CString`] without copying or
    /// allocating.
    #[inline]
    fn from(s: Box<CStr>) -> Self {
        s.into_c_string()
    }
}

impl ops::Index<ops::RangeFull> for CString {
    type Output = CStr;

    #[inline]
    fn index(&self, _index: ops::RangeFull) -> &CStr {
        self
    }
}

impl AsRef<CStr> for CString {
    #[inline]
    fn as_ref(&self) -> &CStr {
        self
    }
}

impl fmt::Display for NulError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "nul byte found in provided data at position: {}", self.0)
    }
}

impl fmt::Display for IntoStringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "C string contained non-utf8 bytes")
    }
}
