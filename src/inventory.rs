//! A zero-cost abstraction to track various resource states with the
//! type-system.
//!
//! Lets describe the pattern by example. (Familiarity with
//! [`token`](crate::token) module may be required.) Imagine that we need to
//! implement a DMA driver. The DMA peripheral consists of the common
//! functionality, which includes the power switch for the whole peripheral, and
//! separate DMA channels. The channels can be used independently in different
//! threads. We want to avoid situations where one thread holding the switch
//! breaks the other thread holding a channel. Lets see an example of the
//! pattern:
//!
//! ```
//! use core::sync::atomic::{AtomicBool, Ordering};
//! use drone_core::{inventory, inventory::Inventory, token::{simple_token, Token}};
//!
//! // Let it be our power switch, so we can easily observe its state.
//! static DMA_EN: AtomicBool = AtomicBool::new(false);
//!
//! // Our drivers map unique resources expressed by tokens.
//! simple_token!(pub struct DmaReg);
//! simple_token!(pub struct DmaChReg);
//!
//! // We split the DMA driver in two types: one for disabled state, and the
//! // other for enabled state.
//! pub struct Dma(Inventory<DmaEn, 0>);
//! pub struct DmaEn(DmaReg);
//!
//! impl Dma {
//!     // The constructor for the DMA driver. Note that `reg` is a token, so at most
//!     // one instance of the driver could ever exist.
//!     pub fn new(reg: DmaReg) -> Self {
//!         Self(Inventory::new(DmaEn(reg)))
//!     }
//!
//!     // It is always a good idea to provide a method to free the token passed to
//!     // the `new()` method above.
//!     pub fn free(self) -> DmaReg {
//!         Inventory::free(self.0).0
//!     }
//!
//!     // This method takes `self` by reference and returns a scoped guard object. It
//!     // enables DMA, and the returned guard will automatically disable it on `drop`.
//!     pub fn enable(&mut self) -> inventory::Guard<'_, DmaEn> {
//!         self.setup();
//!         Inventory::guard(&mut self.0)
//!     }
//!
//!     // This method takes `self` by value and returns the inventory object with one
//!     // token taken. It enables DMA, and in order to disable it, one should
//!     // explicitly call  `from_enabled()` method below.
//!     pub fn into_enabled(self) -> Inventory<DmaEn, 1> {
//!         self.setup();
//!         let (enabled, token) = self.0.share1();
//!         // To be recreated in `from_enabled()`.
//!         drop(token);
//!         enabled
//!     }
//!
//!     // This method takes the inventory object with one token taken, restores the
//!     // token, and disables DMA.
//!     pub fn from_enabled(enabled: Inventory<DmaEn, 1>) -> Self {
//!         // Restoring the token dropped in `into_enabled()`.
//!         let token = unsafe { inventory::Token::new() };
//!         let mut enabled = enabled.merge1(token);
//!         Inventory::teardown(&mut enabled);
//!         Self(enabled)
//!     }
//!
//!     // An example method, which can be called only when DMA is disabled.
//!     pub fn do_something_with_disabled_dma(&self) {}
//!
//!     // A private method that actually enables DMA.
//!     fn setup(&self) {
//!         DMA_EN.store(true, Ordering::Relaxed);
//!     }
//! }
//!
//! impl inventory::Item for DmaEn {
//!     // A method that disables DMA. Due to its signature it can't be called directly.
//!     // It is called only by `Guard::drop` or `Inventory::teardown`.
//!     fn teardown(&mut self, _token: &mut inventory::GuardToken<DmaEn>) {
//!         DMA_EN.store(false, Ordering::Relaxed);
//!     }
//! }
//!
//! impl DmaEn {
//!     // An example method, which can be called only when DMA is enabled.
//!     fn do_something_with_enabled_dma(&self) {}
//! }
//!
//! // Here we define types for DMA channels.
//! pub struct DmaCh(DmaChEn);
//! pub struct DmaChEn(DmaChReg);
//!
//! impl DmaCh {
//!     // The following two methods are the usual constructor and destructor.
//!
//!     pub fn new(reg: DmaChReg) -> Self {
//!         Self(DmaChEn(reg))
//!     }
//!
//!     pub fn free(self) -> DmaChReg {
//!         (self.0).0
//!     }
//!
//!     // A DMA channel is enabled when the whole DMA is enabled. If we have a token
//!     // reference, we can safely assume that the channel is already enabled.
//!
//!     pub fn as_enabled(&self, _token: &inventory::Token<DmaEn>) -> &DmaChEn {
//!         &self.0
//!     }
//!
//!     pub fn as_enabled_mut(&mut self, _token: &inventory::Token<DmaEn>) -> &mut DmaChEn {
//!         &mut self.0
//!     }
//!
//!     // If we consume the token, we can assume that the DMA will be enabled
//!     // infinitely. Or at least until the token will be resurrected.
//!     pub fn into_enabled(self, token: inventory::Token<DmaEn>) -> DmaChEn {
//!         // To be recreated in `into_disabled()`.
//!         drop(token);
//!         self.0
//!     }
//! }
//!
//! impl DmaChEn {
//!     // The only way to obtain an instance of `DmaChEn` is with `DmaCh::into_enabled`
//!     // method. So we can claim that the newly created token is the token dropped in
//!     // `DmaCh::into_enabled`.
//!     pub fn into_disabled(self) -> (DmaCh, inventory::Token<DmaEn>) {
//!         // Restore the token dropped in `into_enabled()`.
//!         let token = unsafe { inventory::Token::new() };
//!         (DmaCh(self), token)
//!     }
//!
//!     // An example method, which can be called only when DMA channel is enabled.
//!     fn do_something_with_enabled_dma_ch(&self) {}
//! }
//!
//! fn main() {
//!     // Instantiate the tokens. This is `unsafe` because we can accidentally
//!     // create more than one instance of a token.
//!     let dma_reg = unsafe { DmaReg::take() };
//!     let dma_ch_reg = unsafe { DmaChReg::take() };
//!
//!     // Instantiate drivers. Only one instance of each driver can exist, because
//!     // they depend on the tokens.
//!     let mut dma = Dma::new(dma_reg);
//!     let mut dma_ch = DmaCh::new(dma_ch_reg);
//!     // DMA is disabled now.
//!     assert!(!DMA_EN.load(Ordering::Relaxed));
//!
//!     // We can call methods defined for disabled `Dma`.
//!     dma.do_something_with_disabled_dma();
//!     // We can't call methods defined for enabled `Dma`. This won't compile.
//!     // dma.do_something_with_enabled_dma();
//!
//!     {
//!         // Enable DMA. This method returns a guard scoped to the enclosing block.
//!         let mut dma = dma.enable();
//!         assert!(DMA_EN.load(Ordering::Relaxed));
//!
//!         // We can call methods defined for enabled DMA.
//!         dma.do_something_with_enabled_dma();
//!         // Calls to methods defined for disabled DMA won't compile.
//!         // dma.do_something_with_disabled_dma();
//!
//!         // Get enabled DMA channel. Type system ensures that the lifetime of
//!         // `dma_ch` is always shorter than the lifetime of `dma`.
//!         let dma_ch = dma_ch.as_enabled(dma.inventory_token());
//!         // We can call methods defined for enabled DMA channel.
//!         dma_ch.do_something_with_enabled_dma_ch();
//!     }
//!     // After exiting the scope above, DMA is automatically disabled.
//!     assert!(!DMA_EN.load(Ordering::Relaxed));
//!
//!     // Sometimes we can't use lifetimes to encode resource states. Here is another
//!     // approach which encodes states in the types.
//!
//!     // Enable DMA while converting our driver to a different type.
//!     let mut dma = dma.into_enabled();
//!     assert!(DMA_EN.load(Ordering::Relaxed));
//!
//!     // We can call methods defined for enabled types.
//!     dma.do_something_with_enabled_dma();
//!     dma_ch
//!         .as_enabled(dma.inventory_token())
//!         .do_something_with_enabled_dma_ch();
//!
//!     // Obtain the owned token from `dma`. From now `dma` has a type that can't be
//!     // disabled.
//!     let (dma, token) = dma.share1();
//!     // Get enabled DMA channel. This method consumes the token.
//!     let dma_ch = dma_ch.into_enabled(token);
//!     // We can call methods defined for enabled DMA channel.
//!     dma_ch.do_something_with_enabled_dma_ch();
//!
//!     // At this moment DMA can't be disabled. If `dma` is dropped, then the
//!     // resource will remain enabled. We need to get our token back from `dma_ch`.
//!     let (dma_ch, token) = dma_ch.into_disabled();
//!     let dma = dma.merge1(token);
//!     // Now DMA can be disabled.
//!     let dma = Dma::from_enabled(dma);
//!     assert!(!DMA_EN.load(Ordering::Relaxed));
//! }
//! ```

use core::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

/// The inventory wrapper for `T`. Parameter `C` encodes the number of emitted
/// tokens.
///
/// See [the module-level documentation](self) for details.
#[repr(transparent)]
pub struct Inventory<T: Item, const C: usize> {
    item: T,
}

/// An RAII scoped guard for the inventory item `T`. Will call
/// [`Item::teardown`] on `drop`.
#[must_use = "if unused the item will immediately teardown"]
pub struct Guard<'a, T: Item> {
    borrow: &'a mut T,
    token: Token<T>,
    guard_token: GuardToken<T>,
}

/// A zero-sized token for [`Item::teardown`]. Cannot be created by the user.
pub struct GuardToken<T: Item>(PhantomData<T>);

/// A zero-sized token for resource `T`. Having an instance or reference to it,
/// guarantees that `T` is in its active state.
pub struct Token<T: Item>(PhantomData<T>);

/// An inventory item interface.
pub trait Item: Sized {
    /// Sets the inactive state. Called by [`Guard`] on `drop`.
    fn teardown(&mut self, _token: &mut GuardToken<Self>);
}

impl<T: Item> Inventory<T, 0> {
    /// Creates a new [`Inventory`] in the inactive state with zero tokens
    /// emitted.
    ///
    /// `item` should contain some form of token.
    #[inline]
    pub fn new(item: T) -> Self {
        Self { item }
    }

    /// Drops `inventory` and returns the stored item.
    #[inline]
    pub fn free(inventory: Self) -> T {
        inventory.item
    }

    /// Creates an RAII scoped guard.
    ///
    /// The item should be already in its active state. The returned guard will
    /// call [`Item::teardown`] on drop.
    #[inline]
    pub fn guard(inventory: &mut Self) -> Guard<'_, T> {
        Guard {
            borrow: &mut inventory.item,
            token: Token(PhantomData),
            guard_token: GuardToken(PhantomData),
        }
    }

    /// Calls [`Item::teardown`] for the stored item.
    #[inline]
    pub fn teardown(inventory: &mut Self) {
        inventory.item.teardown(&mut GuardToken(PhantomData));
    }
}

impl<T: Item, const C: usize> Inventory<T, C> {
    /// Returns a reference to [`Token`]`<T>`. While the reference exists, the
    /// item is always in its active state.
    #[allow(clippy::unused_self)]
    #[inline]
    pub fn inventory_token(&self) -> &Token<T> {
        &Token(PhantomData)
    }
}

macro_rules! define_methods {
    (
        $($share:ident $inc:expr => $($share_token:ident)*;)*
        ;
        $($merge:ident $dec:expr => $($merge_token:ident)*;)*
    ) => {
        impl<T: Item, const C: usize> Inventory<T, C> {
            $(
                /// Returns a token and a new inventory object with increased
                /// counter in its type.
                pub fn $share(self) -> (Inventory<T, { C + $inc }>, $(Token<$share_token>),*) {
                    (Inventory { item: self.item }, $(Token(PhantomData::<$share_token>),)*)
                }
            )*
            $(
                /// Consumes a token and returns a new inventory object with
                /// decreased counter in its type.
                #[allow(clippy::too_many_arguments)]
                pub fn $merge(self, $($merge_token: Token<T>,)*) -> Inventory<T, { C - $dec }> {
                    $(drop($merge_token);)*
                    Inventory { item: self.item }
                }
            )*
        }
    };
}

define_methods! {
    share1 1 => T;
    share2 2 => T T;
    share3 3 => T T T;
    share4 4 => T T T T;
    share5 5 => T T T T T;
    share6 6 => T T T T T T;
    share7 7 => T T T T T T T;
    share8 8 => T T T T T T T T;
    ;
    merge1 1 => a;
    merge2 2 => a b;
    merge3 3 => a b c;
    merge4 4 => a b c d;
    merge5 5 => a b c d e;
    merge6 6 => a b c d e f;
    merge7 7 => a b c d e f g;
    merge8 8 => a b c d e f g h;
}

impl<T: Item, const C: usize> Deref for Inventory<T, C> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        &self.item
    }
}

impl<T: Item, const C: usize> DerefMut for Inventory<T, C> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        &mut self.item
    }
}

impl<T: Item> Token<T> {
    /// Creates a new [`Token`].
    ///
    /// # Safety
    ///
    /// Calling this method is dangerous because it may break the tokens
    /// counting.
    #[inline]
    pub unsafe fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T: Item> Guard<'_, T> {
    /// Returns a reference to [`Token`]`<T>`. While the reference exists, the
    /// item is always in its active state.
    #[inline]
    pub fn inventory_token(&self) -> &Token<T> {
        &self.token
    }
}

impl<T: Item> Deref for Guard<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        self.borrow
    }
}

impl<T: Item> DerefMut for Guard<'_, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        self.borrow
    }
}

impl<T: Item> Drop for Guard<'_, T> {
    #[inline]
    fn drop(&mut self) {
        self.borrow.teardown(&mut self.guard_token);
    }
}
