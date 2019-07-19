use crate::ffi::{c_char, strlen, CString};
use alloc::{borrow::Cow, rc::Rc, sync::Arc};
use core::{
    ascii,
    cmp::Ordering,
    fmt::{self, Write},
    slice::{self, memchr},
    str::{self, Utf8Error},
};

/// Representation of a borrowed C string.
///
/// This type represents a borrowed reference to a nul-terminated array of
/// bytes. It can be constructed safely from a `&[u8]` slice, or unsafely from a
/// raw `*const c_char`. It can then be converted to a Rust `&str` by performing
/// UTF-8 validation, or into an owned [`CString`].
///
/// `CStr` is to [`CString`] as `&str` is to `String`: the former in each pair
/// are borrowed references; the latter are owned strings.
///
/// Note that this structure is **not** `repr(C)` and is not recommended to be
/// placed in the signatures of FFI functions. Instead, safe wrappers of FFI
/// functions may leverage the unsafe [`from_ptr`] constructor to provide a safe
/// interface to other consumers.
///
/// # Examples
///
/// Inspecting a foreign C string:
///
/// ```
/// use drone_core::ffi::{c_char, CStr};
///
/// unsafe fn my_string() -> *const c_char {
///     "foo".as_ptr()
/// }
///
/// unsafe {
///     let slice = CStr::from_ptr(my_string());
///     println!(
///         "string buffer size without nul terminator: {}",
///         slice.to_bytes().len(),
///     );
/// }
/// ```
///
/// Passing a Rust-originating C string:
///
/// ```
/// use drone_core::ffi::{c_char, CStr, CString};
///
/// fn work(data: &CStr) {
///     unsafe fn work_with(_data: *const c_char) {}
///
///     unsafe { work_with(data.as_ptr()) }
/// }
///
/// let s = CString::new("data data data data").unwrap();
/// work(&s);
/// ```
///
/// Converting a foreign C string into a Rust `String`:
///
/// ```
/// use drone_core::ffi::{c_char, CStr};
///
/// unsafe fn my_string() -> *const c_char {
///     "foo".as_ptr()
/// }
///
/// fn my_string_safe() -> String {
///     unsafe { CStr::from_ptr(my_string()).to_string_lossy().into_owned() }
/// }
///
/// println!("string: {}", my_string_safe());
/// ```
///
/// [`CString`]: CString
/// [`from_ptr`]: CStr::from_ptr
#[allow(clippy::derive_hash_xor_eq)]
#[derive(Hash)]
pub struct CStr {
    inner: [c_char],
}

/// An error indicating that a nul byte was not in the expected position.
///
/// The slice used to create a [`CStr`] must have one and only one nul byte at
/// the end of the slice.
///
/// This error is created by the
/// [`from_bytes_with_nul`][`CStr::from_bytes_with_nul`] method on [`CStr`]. See
/// its documentation for more.
///
/// [`CStr`]: CStr
/// [`CStr::from_bytes_with_nul`]: CStr::from_bytes_with_nul
///
/// # Examples
///
/// ```
/// use drone_core::ffi::{CStr, FromBytesWithNulError};
///
/// let _: FromBytesWithNulError = CStr::from_bytes_with_nul(b"f\0oo").unwrap_err();
/// ```
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FromBytesWithNulError {
    kind: FromBytesWithNulErrorKind,
}

#[derive(Clone, PartialEq, Eq, Debug)]
enum FromBytesWithNulErrorKind {
    InteriorNul(usize),
    NotNulTerminated,
}

impl CStr {
    /// Wraps a raw C string with a safe C string wrapper.
    ///
    /// This function will wrap the provided `ptr` with a `CStr` wrapper, which
    /// allows inspection and interoperation of non-owned C strings. This method
    /// is unsafe for a number of reasons:
    ///
    /// * There is no guarantee to the validity of `ptr`.
    /// * The returned lifetime is not guaranteed to be the actual lifetime of
    ///   `ptr`.
    /// * There is no guarantee that the memory pointed to by `ptr` contains a
    ///   valid nul terminator byte at the end of the string.
    /// * It is not guaranteed that the memory pointed by `ptr` won't change
    ///   before the `CStr` has been destroyed.
    ///
    /// > **Note**: This operation is intended to be a 0-cost cast but it is
    /// > currently implemented with an up-front calculation of the length of
    /// > the string. This is not guaranteed to always be the case.
    ///
    /// # Examples
    ///
    /// ```
    /// use drone_core::ffi::{c_char, CStr};
    ///
    /// unsafe fn my_string() -> *const c_char {
    ///     "foo\0".as_ptr()
    /// }
    ///
    /// unsafe {
    ///     let slice = CStr::from_ptr(my_string());
    ///     println!("string returned: {}", slice.to_str().unwrap());
    /// }
    /// ```
    pub unsafe fn from_ptr<'a>(ptr: *const c_char) -> &'a Self {
        let len = strlen(ptr);
        let ptr = ptr as *const u8;
        Self::from_bytes_with_nul_unchecked(slice::from_raw_parts(ptr, len as usize + 1))
    }

    /// Creates a C string wrapper from a byte slice.
    ///
    /// This function will cast the provided `bytes` to a `CStr` wrapper after
    /// ensuring that the byte slice is nul-terminated and does not contain any
    /// interior nul bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use drone_core::ffi::CStr;
    ///
    /// let cstr = CStr::from_bytes_with_nul(b"hello\0");
    /// assert!(cstr.is_ok());
    /// ```
    ///
    /// Creating a `CStr` without a trailing nul terminator is an error:
    ///
    /// ```
    /// use drone_core::ffi::CStr;
    ///
    /// let c_str = CStr::from_bytes_with_nul(b"hello");
    /// assert!(c_str.is_err());
    /// ```
    ///
    /// Creating a `CStr` with an interior nul byte is an error:
    ///
    /// ```
    /// use drone_core::ffi::CStr;
    ///
    /// let c_str = CStr::from_bytes_with_nul(b"he\0llo\0");
    /// assert!(c_str.is_err());
    /// ```
    pub fn from_bytes_with_nul(bytes: &[u8]) -> Result<&Self, FromBytesWithNulError> {
        let nul_pos = memchr::memchr(0, bytes);
        if let Some(nul_pos) = nul_pos {
            if nul_pos + 1 != bytes.len() {
                return Err(FromBytesWithNulError::interior_nul(nul_pos));
            }
            Ok(unsafe { Self::from_bytes_with_nul_unchecked(bytes) })
        } else {
            Err(FromBytesWithNulError::not_nul_terminated())
        }
    }

    /// Unsafely creates a C string wrapper from a byte slice.
    ///
    /// This function will cast the provided `bytes` to a `CStr` wrapper without
    /// performing any sanity checks. The provided slice **must** be
    /// nul-terminated and not contain any interior nul bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use drone_core::ffi::{CStr, CString};
    ///
    /// unsafe {
    ///     let cstring = CString::new("hello").unwrap();
    ///     let cstr = CStr::from_bytes_with_nul_unchecked(cstring.to_bytes_with_nul());
    ///     assert_eq!(cstr, &*cstring);
    /// }
    /// ```
    #[inline]
    pub unsafe fn from_bytes_with_nul_unchecked(bytes: &[u8]) -> &Self {
        &*(bytes as *const [u8] as *const Self)
    }

    /// Returns the inner pointer to this C string.
    ///
    /// The returned pointer will be valid for as long as `self` is, and points
    /// to a contiguous region of memory terminated with a 0 byte to
    /// represent the end of the string.
    ///
    /// **WARNING**
    ///
    /// It is your responsibility to make sure that the underlying memory is not
    /// freed too early. For example, the following code will cause undefined
    /// behavior when `ptr` is used inside the `unsafe` block:
    ///
    /// ```no_run
    /// # #![allow(unused_must_use)]
    /// use drone_core::ffi::CString;
    ///
    /// let ptr = CString::new("Hello").unwrap().as_ptr();
    /// unsafe {
    ///     // `ptr` is dangling
    ///     *ptr;
    /// }
    /// ```
    ///
    /// This happens because the pointer returned by `as_ptr` does not carry any
    /// lifetime information and the [`CString`] is deallocated immediately
    /// after the `CString::new("Hello").unwrap().as_ptr()` expression is
    /// evaluated.  To fix the problem, bind the `CString` to a local
    /// variable:
    ///
    /// ```no_run
    /// # #![allow(unused_must_use)]
    /// use drone_core::ffi::CString;
    ///
    /// let hello = CString::new("Hello").unwrap();
    /// let ptr = hello.as_ptr();
    /// unsafe {
    ///     // `ptr` is valid because `hello` is in scope
    ///     *ptr;
    /// }
    /// ```
    ///
    /// This way, the lifetime of the `CString` in `hello` encompasses the
    /// lifetime of `ptr` and the `unsafe` block.
    ///
    /// [`CString`]: CString
    #[inline]
    pub fn as_ptr(&self) -> *const c_char {
        self.inner.as_ptr()
    }

    /// Converts this C string to a byte slice.
    ///
    /// The returned slice will **not** contain the trailing nul terminator that
    /// this C string has.
    ///
    /// > **Note**: This method is currently implemented as a constant-time
    /// cast, > but it is planned to alter its definition in the future to
    /// perform the > length calculation whenever this method is called.
    ///
    /// # Examples
    ///
    /// ```
    /// use drone_core::ffi::CStr;
    ///
    /// let c_str = CStr::from_bytes_with_nul(b"foo\0").unwrap();
    /// assert_eq!(c_str.to_bytes(), b"foo");
    /// ```
    #[inline]
    pub fn to_bytes(&self) -> &[u8] {
        let bytes = self.to_bytes_with_nul();
        &bytes[..bytes.len() - 1]
    }

    /// Converts this C string to a byte slice containing the trailing 0 byte.
    ///
    /// This function is the equivalent of [`to_bytes`] except that it will
    /// retain the trailing nul terminator instead of chopping it off.
    ///
    /// > **Note**: This method is currently implemented as a 0-cost cast, but
    /// it > is planned to alter its definition in the future to perform the
    /// length > calculation whenever this method is called.
    ///
    /// [`to_bytes`]: CStr::to_bytes
    ///
    /// # Examples
    ///
    /// ```
    /// use drone_core::ffi::CStr;
    ///
    /// let c_str = CStr::from_bytes_with_nul(b"foo\0").unwrap();
    /// assert_eq!(c_str.to_bytes_with_nul(), b"foo\0");
    /// ```
    #[inline]
    pub fn to_bytes_with_nul(&self) -> &[u8] {
        unsafe { &*(&self.inner as *const [c_char] as *const [u8]) }
    }

    /// Yields a `&str` slice if the `CStr` contains valid UTF-8.
    ///
    /// If the contents of the `CStr` are valid UTF-8 data, this function will
    /// return the corresponding `&str` slice. Otherwise, it will return an
    /// error with details of where UTF-8 validation failed.
    ///
    /// > **Note**: This method is currently implemented to check for validity
    /// > after a constant-time cast, but it is planned to alter its definition
    /// in > the future to perform the length calculation in addition to the
    /// UTF-8 > check whenever this method is called.
    ///
    /// # Examples
    ///
    /// ```
    /// use drone_core::ffi::CStr;
    ///
    /// let c_str = CStr::from_bytes_with_nul(b"foo\0").unwrap();
    /// assert_eq!(c_str.to_str(), Ok("foo"));
    /// ```
    pub fn to_str(&self) -> Result<&str, Utf8Error> {
        // NB: When CStr is changed to perform the length check in .to_bytes()
        // instead of in from_ptr(), it may be worth considering if this should
        // be rewritten to do the UTF-8 check inline with the length calculation
        // instead of doing it afterwards.
        str::from_utf8(self.to_bytes())
    }

    /// Converts a `CStr` into a `Cow<str>`.
    ///
    /// If the contents of the `CStr` are valid UTF-8 data, this function will
    /// return a `Cow::Borrowed(&str)` with the the corresponding `&str` slice.
    /// Otherwise, it will replace any invalid UTF-8 sequences with `U+FFFD
    /// REPLACEMENT CHARACTER` and return a `Cow::Owned(String)` with the
    /// result.
    ///
    /// > **Note**: This method is currently implemented to check for validity
    /// > after a constant-time cast, but it is planned to alter its definition
    /// in > the future to perform the length calculation in addition to the
    /// UTF-8 > check whenever this method is called.
    ///
    /// # Examples
    ///
    /// Calling `to_string_lossy` on a `CStr` containing valid UTF-8:
    ///
    /// ```
    /// # #![feature(alloc)]
    /// # extern crate alloc;
    /// use alloc::borrow::Cow;
    /// use drone_core::ffi::CStr;
    ///
    /// let c_str = CStr::from_bytes_with_nul(b"Hello World\0").unwrap();
    /// assert_eq!(c_str.to_string_lossy(), Cow::Borrowed("Hello World"));
    /// ```
    ///
    /// Calling `to_string_lossy` on a `CStr` containing invalid UTF-8:
    ///
    /// ```
    /// # #![feature(alloc)]
    /// # extern crate alloc;
    /// use alloc::borrow::Cow;
    /// use drone_core::ffi::CStr;
    ///
    /// let c_str = CStr::from_bytes_with_nul(b"Hello \xF0\x90\x80World\0").unwrap();
    /// assert_eq!(
    ///     c_str.to_string_lossy(),
    ///     Cow::Owned(String::from("Hello �World")) as Cow<str>
    /// );
    /// ```
    pub fn to_string_lossy(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(self.to_bytes())
    }

    /// Converts a `Box<CStr>` into a [`CString`] without copying or allocating.
    ///
    /// [`CString`]: CString
    ///
    /// # Examples
    ///
    /// ```
    /// use drone_core::ffi::CString;
    ///
    /// let c_string = CString::new(b"foo".to_vec()).unwrap();
    /// let boxed = c_string.into_boxed_c_str();
    /// assert_eq!(boxed.into_c_string(), CString::new("foo").unwrap());
    /// ```
    #[allow(clippy::wrong_self_convention)]
    pub fn into_c_string(self: Box<Self>) -> CString {
        let raw = Box::into_raw(self) as *mut [u8];
        CString {
            inner: unsafe { Box::from_raw(raw) },
        }
    }
}

impl FromBytesWithNulError {
    fn interior_nul(pos: usize) -> Self {
        Self {
            kind: FromBytesWithNulErrorKind::InteriorNul(pos),
        }
    }

    fn not_nul_terminated() -> Self {
        Self {
            kind: FromBytesWithNulErrorKind::NotNulTerminated,
        }
    }
}

impl fmt::Debug for CStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"")?;
        for byte in self
            .to_bytes()
            .iter()
            .flat_map(|&b| ascii::escape_default(b))
        {
            f.write_char(byte as char)?;
        }
        write!(f, "\"")
    }
}

impl<'a> Default for &'a CStr {
    fn default() -> &'a CStr {
        const SLICE: &[c_char] = &[0];
        unsafe { CStr::from_ptr(SLICE.as_ptr()) }
    }
}

impl PartialEq for CStr {
    fn eq(&self, other: &Self) -> bool {
        self.to_bytes().eq(other.to_bytes())
    }
}

impl Eq for CStr {}

impl PartialOrd for CStr {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.to_bytes().partial_cmp(other.to_bytes())
    }
}

impl Ord for CStr {
    fn cmp(&self, other: &Self) -> Ordering {
        self.to_bytes().cmp(other.to_bytes())
    }
}

impl ToOwned for CStr {
    type Owned = CString;

    fn to_owned(&self) -> CString {
        CString {
            inner: self.to_bytes_with_nul().into(),
        }
    }
}

impl AsRef<CStr> for CStr {
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<'a> From<&'a CStr> for Box<CStr> {
    fn from(s: &'a CStr) -> Self {
        let boxed: Box<[u8]> = Box::from(s.to_bytes_with_nul());
        unsafe { Box::from_raw(Box::into_raw(boxed) as *mut CStr) }
    }
}

impl From<CString> for Box<CStr> {
    #[inline]
    fn from(s: CString) -> Self {
        s.into_boxed_c_str()
    }
}

impl From<CString> for Arc<CStr> {
    #[inline]
    fn from(s: CString) -> Self {
        let arc: Arc<[u8]> = Arc::from(s.into_inner());
        unsafe { Arc::from_raw(Arc::into_raw(arc) as *const CStr) }
    }
}

impl<'a> From<&'a CStr> for Arc<CStr> {
    #[inline]
    fn from(s: &CStr) -> Self {
        let arc: Arc<[u8]> = Arc::from(s.to_bytes_with_nul());
        unsafe { Arc::from_raw(Arc::into_raw(arc) as *const CStr) }
    }
}

impl From<CString> for Rc<CStr> {
    #[inline]
    fn from(s: CString) -> Self {
        let rc: Rc<[u8]> = Rc::from(s.into_inner());
        unsafe { Rc::from_raw(Rc::into_raw(rc) as *const CStr) }
    }
}

impl<'a> From<&'a CStr> for Rc<CStr> {
    #[inline]
    fn from(s: &CStr) -> Self {
        let rc: Rc<[u8]> = Rc::from(s.to_bytes_with_nul());
        unsafe { Rc::from_raw(Rc::into_raw(rc) as *const CStr) }
    }
}

impl<'a> From<CString> for Cow<'a, CStr> {
    #[inline]
    fn from(s: CString) -> Cow<'a, CStr> {
        Cow::Owned(s)
    }
}

impl<'a> From<&'a CStr> for Cow<'a, CStr> {
    #[inline]
    fn from(s: &'a CStr) -> Cow<'a, CStr> {
        Cow::Borrowed(s)
    }
}

impl<'a> From<&'a CString> for Cow<'a, CStr> {
    #[inline]
    fn from(s: &'a CString) -> Cow<'a, CStr> {
        Cow::Borrowed(s.as_c_str())
    }
}

impl Default for Box<CStr> {
    fn default() -> Self {
        let boxed: Box<[u8]> = Box::from([0]);
        unsafe { Box::from_raw(Box::into_raw(boxed) as *mut CStr) }
    }
}

impl fmt::Display for FromBytesWithNulError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl fmt::Display for FromBytesWithNulErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FromBytesWithNulErrorKind::InteriorNul(pos) => write!(
                f,
                "data provided contains an interior nul byte at byte pos {}",
                pos
            ),
            FromBytesWithNulErrorKind::NotNulTerminated => {
                write!(f, "data provided is not nul terminated")
            }
        }
    }
}
