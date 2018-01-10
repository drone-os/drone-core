#![feature(proc_macro)]

extern crate drone_core;

use drone_core::reg::mappings;
use drone_core::reg::prelude::*;

mappings!(FOO; BAR { 0xDEAD_BEEF 0x20 0xBEEF_CACE; BAZ { 0 1 } });

fn assert_send<T: Send>() {}
fn assert_sync<T: Sync>() {}

fn main() {
  assert_send::<foo::Bar<Srt>>();
  assert_sync::<foo::Bar<Srt>>();
  assert_send::<foo::Bar<Frt>>();
  assert_sync::<foo::Bar<Frt>>();
  assert_send::<foo::Bar<Crt>>();
  assert_sync::<foo::Bar<Crt>>();
}
