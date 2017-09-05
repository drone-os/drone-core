//! Instrumentation Trace Macrocell support.

pub use self::port::Port;
use core::fmt::{self, Write};
use drone::reg::{Delegate, ValuePointer};
use reg::{Areg, Reg};
use reg::dbg::{self, ItmtpMask, McucrTraceMode, TpiusppMode};
use util;

const POST_FLUSH_WAIT: u32 = 0x400;

pub mod port;
#[macro_use]
pub mod macros;


/// Initializes ITM.
///
/// # Safety
///
/// Must be called exactly once and as early as possible.
#[cfg_attr(feature = "clippy", allow(too_many_arguments))]
pub unsafe fn init<A, B, C, D, E, F, G>(
  dbg_mcucr: &Reg<dbg::Mcucr, A>,
  dbg_demcr: &Reg<dbg::Demcr, B>,
  dbg_tpiuspp: &Reg<dbg::Tpiuspp, C>,
  dbg_tpiuffc: &Reg<dbg::Tpiuffc, D>,
  dbg_itmla: &Reg<dbg::Itmla, E>,
  dbg_itmtc: &Reg<dbg::Itmtc, F>,
  dbg_itmtp: &Reg<dbg::Itmtp, G>,
) {
  dbg_mcucr
    .ptr()
    .modify(|reg| reg.trace_mode(McucrTraceMode::Async));
  dbg_demcr.ptr().modify(|reg| reg.trace_enable(true));
  dbg_tpiuspp.ptr().write(|reg| reg.mode(TpiusppMode::SwoNrz));
  dbg_tpiuffc.ptr().modify(|reg| reg.formatter_enable(false));
  dbg_itmla.ptr().write(|reg| reg.unlock());
  dbg_itmtc.ptr().modify(|reg| reg.itm_enable(true).atb_id(1));
  dbg_itmtp
    .ptr()
    .modify(|reg| reg.trace_enable(ItmtpMask::P0To7, true));
}

/// Prints `str` to the ITM port #0.
///
/// See [iprint](../macro.iprint.html) and [iprintln](../macro.iprintln.html)
/// macros.
pub fn write_str(string: &str) {
  Port::new(0).write_str(string).unwrap();
}

/// Prints `core::fmt::Arguments` to the ITM port #0.
///
/// See [iprint](../macro.iprint.html) and [iprintln](../macro.iprintln.html)
/// macros.
pub fn write_fmt(args: fmt::Arguments) {
  Port::new(0).write_fmt(args).unwrap();
}

/// Waits until all pending packets will be transmitted.
pub fn flush() {
  let reg: Areg<dbg::Itmtc> = Areg::new();
  let reg = reg.ptr();
  while reg.read().busy() {}
  util::spin(POST_FLUSH_WAIT); // Additional wait due to asynchronous output
}
