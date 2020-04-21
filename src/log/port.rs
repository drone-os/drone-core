use super::{
    drone_log_is_enabled, drone_log_write_bytes, drone_log_write_u16, drone_log_write_u32,
    drone_log_write_u8, PORTS_COUNT,
};
use core::{fmt, fmt::Write};

/// Logger port handle.
#[derive(Clone, Copy)]
pub struct Port(u8);

pub trait PortWrite: Copy {
    fn port_write(port: u8, value: Self);
}

impl Port {
    /// Creates a new port handle.
    ///
    /// # Panics
    ///
    /// If `port` is more than or equal to [`PORTS_COUNT`].
    #[inline]
    pub fn new(port: u8) -> Self {
        assert!(port < PORTS_COUNT);
        Self(port)
    }

    /// Returns `true` if the debug probe is connected and listening to the
    /// `port` stream.
    #[inline]
    pub fn is_enabled(self) -> bool {
        #[cfg(feature = "std")]
        return false;
        let Self(port) = self;
        unsafe { drone_log_is_enabled(port) }
    }

    /// Writes a sequence of bytes to the port.
    ///
    /// The resulting byte sequence that will be read from the port may be
    /// interleaved with concurrent writes. See also [`Port::write`] for writing
    /// atomic byte sequences.
    #[inline]
    pub fn write_bytes(self, bytes: &[u8]) -> Self {
        #[cfg(feature = "std")]
        return self;
        let Self(port) = self;
        unsafe { drone_log_write_bytes(port, bytes.as_ptr(), bytes.len()) };
        self
    }

    /// Writes an atomic byte sequence to the port. `T` can be one of `u8`,
    /// `u16`, `u32`.
    ///
    /// Bytes are written in big-endian order. It's guaranteed that all bytes of
    /// `value` will not be split.
    #[inline]
    pub fn write<T: PortWrite>(self, value: T) -> Self {
        let Self(port) = self;
        T::port_write(port, value);
        self
    }
}

impl Write for Port {
    #[inline]
    fn write_str(&mut self, string: &str) -> fmt::Result {
        self.write_bytes(string.as_bytes());
        Ok(())
    }
}

impl PortWrite for u8 {
    fn port_write(port: u8, value: Self) {
        #[cfg(feature = "std")]
        return;
        unsafe { drone_log_write_u8(port, value) };
    }
}

impl PortWrite for u16 {
    fn port_write(port: u8, value: Self) {
        #[cfg(feature = "std")]
        return;
        unsafe { drone_log_write_u16(port, value) };
    }
}

impl PortWrite for u32 {
    fn port_write(port: u8, value: Self) {
        #[cfg(feature = "std")]
        return;
        unsafe { drone_log_write_u32(port, value) };
    }
}
