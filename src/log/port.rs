use super::{drone_log_is_port_enabled, drone_log_port_write_bytes};
use core::{fmt, fmt::Write};

/// Logger port.
#[derive(Clone, Copy)]
pub struct Port {
    port: u8,
    exclusive: bool,
}

impl Port {
    /// Creates a new port handle.
    #[inline]
    pub const fn new(port: u8) -> Self {
        Self { port, exclusive: false }
    }

    /// Creates a new port handle.
    #[inline]
    pub const fn exclusive(port: u8) -> Self {
        Self { port, exclusive: true }
    }

    /// Returns `true` if the debug probe is connected and listening to the
    /// `port` stream.
    #[inline]
    pub fn is_enabled(self) -> bool {
        #[cfg(feature = "std")]
        return false;
        unsafe { drone_log_is_port_enabled(self.port) }
    }

    /// Writes `bytes` to the port.
    #[inline]
    pub fn write_bytes(&mut self, bytes: &[u8]) {
        #[cfg(feature = "std")]
        return;
        let Self { port, exclusive } = *self;
        unsafe { drone_log_port_write_bytes(port, exclusive, bytes.as_ptr(), bytes.len()) };
    }
}

impl Write for Port {
    #[inline]
    fn write_str(&mut self, string: &str) -> fmt::Result {
        self.write_bytes(string.as_bytes());
        Ok(())
    }
}
