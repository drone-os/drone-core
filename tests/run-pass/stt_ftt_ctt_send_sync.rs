#![feature(proc_macro)]

extern crate drone_core;

use drone_core::reg::mappings;
use drone_core::reg::prelude::*;

mappings!(FOO; BAR { 0xDEAD_BEEF 0x20 0xBEEF_CACE; BAZ { 0 1 } });

fn assert_send<T: Send>() {}
fn assert_sync<T: Sync>() {}

fn main() {
  assert_send::<foo::Bar<Stt>>();
  assert_sync::<foo::Bar<Stt>>();
  assert_send::<foo::Bar<Ftt>>();
  assert_sync::<foo::Bar<Ftt>>();
  assert_send::<foo::Bar<Ctt>>();
  assert_sync::<foo::Bar<Ctt>>();
}
