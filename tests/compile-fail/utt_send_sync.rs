#![feature(proc_macro)]

extern crate drone_core;

use drone_core::reg::mappings;
use drone_core::reg::prelude::*;

mappings!(FOO; BAR { 0xDEAD_BEEF 0x20 0xBEEF_CACE BAZ { 0 1 } });

fn assert_send<T: Send>() {}
fn assert_sync<T: Sync>() {}

fn main() {
  assert_send::<foo::Bar<Utt>>();
  assert_sync::<foo::Bar<Utt>>();
  //~^ ERROR `drone_core::reg::Utt: std::marker::Sync` is not satisfied
}
