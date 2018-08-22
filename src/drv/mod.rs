//! Device drivers.
//!
//! ```
//! # #![feature(extern_in_paths)]
//! # extern crate std as core;
//! # #[macro_use] extern crate drone_core;
//! use core::cell::RefCell;
//! use drone_core::drv::Driver;
//!
//! #[derive(Driver)]
//! #[driver(forward)]
//! struct A(B);
//!
//! #[derive(Driver, Resource)]
//! #[driver(forward)]
//! struct B(Option<C>);
//!
//! #[derive(Driver, Resource)]
//! #[driver(forward)]
//! struct C(RefCell<D>);
//!
//! #[derive(Driver, Resource)]
//! struct D(RefCell<Option<E>>);
//!
//! #[derive(Resource)]
//! struct E;
//!
//! # fn main() {
//! let a: A = A::new(E);
//! let e: E = a.free();
//! # }
//! ```

mod driver;
mod macros;
mod resource;

pub use self::driver::Driver;
pub use self::resource::Resource;
