//! The [`Token`][`token::Token`] trait and common patterns.
//!
//! This module is centered around the [`Token`][`token::Token`] trait - a
//! common pattern in Drone for zero-sized singletons. Tokens represent
//! ownerships of various resources: memory-mapped registers, threads, mutable
//! statics, one-time initializers.
//!
//! Singleton properties can't be declared by type-system, therefore here we
//! rely on `unsafe` contracts. Declaring a token type is often `unsafe` because
//! the developer should ensure that the target resource is not used in other
//! ways than through the token. Instantiating a token object is `unsafe`
//! because only one instance of the token should exist. It is supposed that the
//! tokens are grouped in sets and instantiated as early as possible alongside
//! of other unsafe initializations like [`mem::data_init`] or
//! [`heap::Allocator::init`].
//!
//! # Init Tokens
//!
//! Sometimes we need to ensure that a particular function is executed only once
//! in the entire program lifetime. Here is an example:
//!
//! ```
//! use drone_core::token::{unit_token, unsafe_init_tokens, Token};
//!
//! unit_token! {
//!     /// The token for Foo initializer.
//!     pub struct FooInitToken;
//! }
//!
//! unit_token! {
//!     /// The token for Bar initializer.
//!     pub struct BarInitToken;
//! }
//!
//! // Here is `unsafe`, we need to ensure that `FooInitToken` and `BarInitToken`
//! // are not used anywhere else.
//! unsafe_init_tokens! {
//!     /// The group token for all initializers.
//!     pub struct Inits {
//!         FooInitToken,
//!         BarInitToken,
//!     }
//! }
//!
//! // Define one-time initializers. They should accept tokens by-value and
//! // shouldn't return them.
//!
//! fn init_foo(token: FooInitToken) {
//!     // Initialize Foo.
//! }
//!
//! fn init_bar(token: BarInitToken) {
//!     // Initialize Bar.
//! }
//!
//! // Your entry point.
//! fn main() {
//!     // Various unsafe initializations goes here.
//!     unsafe {
//!         // Calling unsafe take(), we need to ensure that it is the only
//!         // place we call it and we are not in a cycle or recursion.
//!         let ini = Inits::take();
//!         // Pass the token to your safe entry point.
//!         trunk(ini);
//!     }
//! }
//!
//! fn trunk(ini: Inits) {
//!     init_foo(ini.foo_init);
//!     init_bar(ini.bar_init);
//!     // Calling them again won't compile, because the tokens were consumed.
//!     // init_foo(ini.foo_init);
//!     // init_bar(ini.bar_init);
//! }
//! ```
//!
//! # Static Tokens
//!
//! Mutable statics are unsafe in Rust. One way to make them safe is to use the
//! interior-mutability. For example [`Mutex`][`crate::sync::mutex::Mutex`]
//! ensures that concurrent access to the data is safe. If you don't need
//! simultaneous access to the static but still need other static
//! characteristics like known and stable address, you can use static tokens:
//!
//! ```
//! use drone_core::token::{unsafe_static_tokens, StaticToken, Token};
//!
//! // Define some statics.
//! static mut FOO: usize = 0;
//! static mut BAR: &str = "data";
//!
//! // Here is `unsafe`, we need to ensure that `FOO` and `BAR` are not used
//! // anywhere else.
//! unsafe_static_tokens! {
//!     /// The group token for all statics.
//!     pub struct Statics {
//!         FOO: usize,
//!         BAR: &'static str,
//!     }
//! }
//!
//! // Your entry point.
//! fn main() {
//!     // Various unsafe initializations goes here.
//!     unsafe {
//!         // Calling unsafe take(), we need to ensure that it is the only
//!         // place we call it and we are not in a cycle or recursion.
//!         let stc = Statics::take();
//!         // Pass the token to your safe entry point.
//!         trunk(stc);
//!     }
//! }
//!
//! fn trunk(mut stc: Statics) {
//!     // Borrow a mutable reference.
//!     add_one(stc.foo.get());
//!     add_one(stc.foo.get());
//!     assert_eq!(*stc.foo.get(), 2);
//!     assert_eq!(core::mem::size_of_val(&stc), 0);
//!     // Permanently convert to `&'static usize`. Note that `foo` is no longer a ZST.
//!     let foo = stc.foo.into_static();
//!     // Calling it again won't compile, because the token was consumed.
//!     // let foo = stc.foo.into_static();
//! }
//!
//! fn add_one(foo: &mut usize) {
//!     *foo += 1;
//! }
//! ```

/// Defines a new unit token.
///
/// See [the module-level documentation][self] for details.
pub use drone_core_macros::unit_token;

/// Defines a new token for the set of unit tokens.
///
/// See [the module-level documentation][self] for details.
///
/// # Safety
///
/// The inner tokens must not be instantiated elsewhere.
pub use drone_core_macros::unsafe_init_tokens;

/// Defines a new token for the set of [`StaticToken`]s.
///
/// See [the module-level documentation][self] for details.
///
/// # Safety
///
/// The inner statics must not be used directly elsewhere.
pub use drone_core_macros::unsafe_static_tokens;

/// Zero-sized singleton.
///
/// A token is a ZST which assumed to be a singleton. The invariants can't be
/// enforced by the type-system, therefore the trait is marked `unsafe` and the
/// invariants should be maintained manually as part of the `unsafe` contract.
///
/// The caller should assume that the instance of the type is the only instance
/// in the entire lifetime of the program. Because the instance can be
/// constructed only by [`Token::take`] method, which is marked `unsafe`.
///
/// # Safety
///
/// * The type must be instantiated only inside [`Token::take`] method.
/// * The type must be zero-sized.
/// * If the type contains other zero-sized types, they must be instantiated
///   only as part of this type.
pub unsafe trait Token: Sized + Send + 'static {
    /// Creates the only instance of the type. Calling this method more than
    /// once in the entire lifetime of the program violates the contract.
    ///
    /// # Safety
    ///
    /// Must be called no more than once in the entire lifetime of the program.
    unsafe fn take() -> Self;
}

/// A mutable static token.
///
/// See [the module-level documentation][self] for details.
///
/// # Safety
///
/// * The target static must be used only inside [`StaticToken`] methods.
/// * The type, which implements this trait, must not be `Sync`.
pub unsafe trait StaticToken: Token + Sized + Send + 'static {
    /// Type of the target static.
    type Target: ?Sized;

    /// Borrows a mutable reference.
    fn get(&mut self) -> &mut Self::Target;

    /// Converts the token into a mutable reference with `'static` lifetime.
    fn into_static(self) -> &'static mut Self::Target;
}

mod compile_tests {
    //! ```compile_fail
    //! drone_core::token::unit_token!(struct Foo);
    //! fn main() {
    //!     let foo = Foo { __priv: () };
    //! }
    //! ```
    //!
    //! ```compile_fail
    //! use drone_core::token::Token;
    //! drone_core::token::unit_token!(struct FooToken);
    //! drone_core::token::unsafe_init_tokens! {
    //!     struct Foo {
    //!         FooToken,
    //!     }
    //! }
    //! fn main() {
    //!     let foo = unsafe {
    //!         Foo {
    //!             foo: FooToken::take(),
    //!             __priv: (),
    //!         }
    //!     };
    //! }
    //! ```
    //!
    //! ```compile_fail
    //! use drone_core::token::Token;
    //! static mut FOO: usize = 0;
    //! drone_core::token::unsafe_static_tokens! {
    //!     struct Foo {
    //!         FOO: usize,
    //!     }
    //! }
    //! fn main() {
    //!     let foo = unsafe {
    //!         Foo {
    //!             foo: FooToken::take(),
    //!             __priv: (),
    //!         }
    //!     };
    //! }
    //! ```
}
