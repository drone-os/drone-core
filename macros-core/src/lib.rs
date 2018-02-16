//! Drone procedural macros shared lib.
//!
//! See `drone` documentation for details.

#![warn(missing_docs)]
#![doc(html_root_url = "https://docs.rs/drone-macros-core/0.8.0")]
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![cfg_attr(feature = "clippy", allow(precedence))]

#[macro_use]
extern crate failure_dup as failure;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate quote;
extern crate regex;
extern crate syn;

mod reserved;
mod parse_name;

pub use self::parse_name::{parse_extern_name, parse_own_name};
pub use self::reserved::reserved_check;

#[macro_export]
macro_rules! tokens {
  ($tokens:expr) => {
    match $tokens {
      Ok(tokens) => tokens.parse().unwrap(),
      Err(error) => panic!("{}", error),
    }
  }
}
