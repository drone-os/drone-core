//! Device drivers.

pub mod prelude;

mod driver;
mod macros;
mod resource;

pub use self::driver::Driver;
pub use self::resource::Resource;
