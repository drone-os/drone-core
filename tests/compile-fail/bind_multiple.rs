#![feature(decl_macro)]

extern crate drone;

use drone::reg::bind;

fn main() {
  bind!();
  bind!(); //~ ERROR proc macro panicked
}
