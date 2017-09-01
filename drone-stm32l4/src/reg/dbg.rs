//! The processor debug.

use drone_core::reg::{RawBits, RawValue};
use reg::Value;

define_reg! {
  name => Tpiuspp,
  desc => "TPIU selected pin protocol.",
  addr => 0xE004_00F0,
}

define_reg! {
  name => Tpiuffc,
  desc => "TPIU formatter and flush control.",
  addr => 0xE004_0304,
}

define_reg! {
  name => Itmtp,
  desc => "ITM trace privilege.",
  addr => 0xE000_0E40,
}

define_reg! {
  name => Itmtc,
  desc => "ITM trace control.",
  addr => 0xE000_0E80,
}

define_reg! {
  name => Itmla,
  desc => "ITM lock access.",
  addr => 0xE000_0FB0,
}

define_reg! {
  name => Mcucr,
  desc => "Debug MCU configuration register.",
  addr => 0xE004_2004,
}

define_reg! {
  name => Demcr,
  desc => "Debug Exception and Monitor Control Register.",
  addr => 0xE000_EDFC,
}

/// TPIU selected pin protocol mode.
#[repr(u32)]
pub enum TpiusppMode {
  /// Sync Trace Port Mode.
  SyncTrace = 0b00,
  /// Serial Wire Output - manchester **(default value)**.
  SwoManchester = 0b01,
  /// Serial Wire Output - NRZ.
  SwoNrz = 0b10,
}

/// ITM trace privilege port mask.
#[repr(u32)]
pub enum ItmtpMask {
  /// Mask to enable/disable tracing ports 7:0.
  P0To7 = 0,
  /// Mask to enable/disable tracing ports 15:8.
  P8To15 = 1,
  /// Mask to enable/disable tracing ports 23:16.
  P16To23 = 2,
  /// Mask to enable/disable tracing ports 31:24.
  P24To31 = 3,
}

/// Debug MCU configuration register trace mode.
#[repr(u32)]
pub enum McucrTraceMode {
  /// TRACE pins not assigned **(default state)**.
  Disabled = 0b000,
  /// TRACE pin assignment for Asynchronous Mode.
  Async = 0b001,
  /// TRACE pin assignment for Synchronous Mode with a `TRACEDATA` size of `1`.
  Sync1 = 0b011,
  /// TRACE pin assignment for Synchronous Mode with a `TRACEDATA` size of `2`.
  Sync2 = 0b101,
  /// TRACE pin assignment for Synchronous Mode with a `TRACEDATA` size of `4`.
  Sync4 = 0b111,
}

impl Value<Tpiuspp> {
  /// Selects a pin protocol.
  pub fn mode(&mut self, mode: TpiusppMode) -> &mut Value<Tpiuspp> {
    self.set(mode as u32)
  }
}

impl Value<Tpiuffc> {
  /// Sets `EnFCont` bit.
  ///
  /// Activates or deactivates TPIU formatter.
  pub fn formatter_enable(&mut self, enable: bool) -> &mut Value<Tpiuffc> {
    self.write(1, enable)
  }
}

impl Value<Itmtp> {
  /// Enable tracing a range of ports by mask.
  pub fn trace_enable(
    &mut self,
    mask: ItmtpMask,
    enable: bool,
  ) -> &mut Value<Itmtp> {
    self.write(mask as u32, enable)
  }
}

impl Value<Itmtc> {
  /// Sets `ITMENA` bit.
  ///
  /// Global Enable Bit of the ITM.
  pub fn itm_enable(&mut self, enable: bool) -> &mut Value<Itmtc> {
    self.write(0, enable)
  }

  /// Sets ATB ID which identifies the source of the trace data.
  ///
  /// # Panics
  ///
  /// If `id` is greater or equals to `0x80`.
  pub fn atb_id(&mut self, id: u32) -> &mut Value<Itmtc> {
    assert!(id < 0x80);
    self.write_bits(id, 7, 16)
  }

  /// Returns busy status.
  pub fn busy(&self) -> bool {
    self.read(23)
  }
}

impl Value<Itmla> {
  /// Unlocks Write Access to the other ITM registers.
  pub fn unlock(&mut self) -> &mut Value<Itmla> {
    self.set(0xC5AC_CE55)
  }
}

impl Value<Mcucr> {
  /// Trace pin assignment control.
  pub fn trace_mode(&mut self, config: McucrTraceMode) -> &mut Value<Mcucr> {
    self.write_bits(config as u32, 3, 5)
  }
}

impl Value<Demcr> {
  /// Sets `TRCENA` bit.
  ///
  /// Global enable for all DWT and ITM features.
  pub fn trace_enable(&mut self, enable: bool) -> &mut Value<Demcr> {
    self.write(24, enable)
  }
}
