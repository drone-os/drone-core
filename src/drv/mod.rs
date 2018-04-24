//! Device drivers.
//!
//! ```
//! # #![feature(proc_macro)]
//! # #[macro_use] extern crate drone_core;
//! use drone_core::drv::Driver;
//!
//! #[derive(Driver)]
//! #[driver(forward)]
//! struct Foo(Bar);
//!
//! #[derive(Driver, Resource)]
//! struct Bar(Option<Baz>);
//!
//! #[derive(Resource)]
//! struct Baz;
//!
//! # fn main() {
//! let foo: Foo = Foo::new(Baz);
//! let baz: Baz = foo.free();
//! # }
//! ```

mod driver;
mod macros;
mod resource;

pub use self::driver::Driver;
pub use self::resource::Resource;
