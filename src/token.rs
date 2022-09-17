//! The [`Token`] trait and its common patterns.
//!
//! A token is a zero-sized type, at most one instance of which ever exists.
//! This concept is ubiquitous in Drone. It is used for representing
//! memory-mapped registers, threads, one-time initializers, mutable statics
//! ownership. While affinity (also called move-semantics in Rust) could be
//! represented by Rust type-system, the other properties couldn't. Therefore
//! the concept relies on the two `unsafe` contracts below.
//!
//! 1. *Implementing* the trait is `unsafe`, and it is the implementer
//! responsibility to ensure the following:
//!
//!     * The type must not implement [`Clone`].
//!     * The type must be instantiated only inside [`Token::take`] method.
//!     * The type must be zero-sized.
//!
//! 2. *Calling* [`Token::take`] is `unsafe`, and it is the caller
//! responsibility to ensure that at most one instance of the type ever exists.
//!
//! Tokens are often nested to minimize the usage of `unsafe` [`Token::take`]
//! constructor. It is supposed to instantiate all needed tokens at the very
//! beginning of the program and pass the instances further to the code.
//!
//! Since tokens are zero-sized, [`Token::take`] is no-op from the assembly
//! perspective. Likewise passing the instance around doesn't consume the stack,
//! and storing the instance inside other types doesn't consume the memory.
//!
//! # Simple Tokens
//!
//! Here is a usage example of tokens of their simplest form - `simple_token!`
//! macro. In this example we implement one-timer initializers.
//!
//! ```
//! use drone_core::token::{simple_token, unsafe_simple_tokens, Token};
//!
//! simple_token! {
//!     /// The token for Foo initializer.
//!     pub struct FooInitToken;
//! }
//!
//! simple_token! {
//!     /// The token for Bar initializer.
//!     pub struct BarInitToken;
//! }
//!
//! // Here is `unsafe`, we need to ensure that `FooInitToken` and `BarInitToken`
//! // are not used anywhere else.
//! unsafe_simple_tokens! {
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
//!         // Calling unsafe `take()`, we need to ensure that it is the only place
//!         // we call it and we are not in a cycle or recursion.
//!         let ini = Inits::take();
//!         // Pass the token instance to your safe entry point.
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
//! Mutable statics are unsafe in Rust. One way to make them safe is to use
//! interior-mutability. For example [`Mutex`](crate::sync::Mutex) ensures
//! that concurrent access to the data is safe. If you don't need simultaneous
//! access to the static but still need other static characteristics like known
//! and stable address, you can use static tokens:
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
//!         // Calling unsafe `take()`, we need to ensure that it is the only place
//!         // we call it and we are not in a cycle or recursion.
//!         let stc = Statics::take();
//!         // Pass the token instance to your safe entry point.
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

/// Defines a new simple [`Token`].
///
/// See [the module-level documentation](self) for details.
#[doc(inline)]
pub use drone_core_macros::simple_token;

/// Defines a new token for the set of simple [`Token`]s.
///
/// See [the module-level documentation](self) for details.
///
/// # Safety
///
/// The tokens must not be instantiated anywhere else.
#[doc(inline)]
pub use drone_core_macros::unsafe_simple_tokens;

/// Defines a new token for the set of [`StaticToken`]s.
///
/// See [the module-level documentation](self) for details.
///
/// # Safety
///
/// The tokens must not be instantiated anywhere else.
#[doc(inline)]
pub use drone_core_macros::unsafe_static_tokens;

/// A zero-sized affine type, at most one instance of which ever exists.
///
/// # Safety
///
/// The above properties can't be expressed with Rust type-system, therefore
/// the trait is marked `unsafe`, and it is the implementer's responsibility to
/// keep the following invariants:
///
/// 1. The type must not implement [`Clone`].
/// 2. The type must be instantiated only inside [`Token::take`] method.
/// 3. The type must be zero-sized.
pub unsafe trait Token: Sized + Send + 'static {
    /// Creates the token instance.
    ///
    /// # Safety
    ///
    /// At most one instance of the token must ever exist. This invariant can't
    /// be expressed with Rust type-system, therefore the method is marked
    /// `unsafe`, and it is the caller responsibility to keep the invariant.
    ///
    /// It is recommended to call this method at the very beginning of the
    /// program and pass the instance further to the code.
    ///
    /// Since the type is ZST, the method is no-op from the assembly
    /// perspective. Likewise passing the instance around doesn't consume
    /// the stack, and storing the instance inside other types doesn't
    /// consume the memory.
    unsafe fn take() -> Self;
}

/// A token for a mutable static variable.
///
/// See [the module-level documentation](self) for details.
///
/// # Safety
///
/// * The type must not implement [`Sync`].
/// * The target static must not be used anywhere else.
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
    //! drone_core::token::simple_token!(struct Foo);
    //! fn main() {
    //!     let foo = Foo { __priv: () };
    //! }
    //! ```
    //!
    //! ```compile_fail
    //! use drone_core::token::Token;
    //! drone_core::token::simple_token!(struct FooToken);
    //! drone_core::token::unsafe_simple_tokens! {
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
    //!     let foo = unsafe { Foo { foo: FooToken::take(), __priv: () } };
    //! }
    //! ```
}
