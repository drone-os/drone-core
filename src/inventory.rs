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
//! use drone_core::{
//!     inventory::{self, Inventory0, Inventory1},
//!     token::{simple_token, Token},
//! };
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
//! pub struct Dma(Inventory0<DmaEn>);
//! pub struct DmaEn(DmaReg);
//!
//! impl Dma {
//!     // The constructor for the DMA driver. Note that `reg` is a token, so at most
//!     // one instance of the driver could ever exist.
//!     pub fn new(reg: DmaReg) -> Self {
//!         Self(Inventory0::new(DmaEn(reg)))
//!     }
//!
//!     // It is always a good idea to provide a method to free the token passed to
//!     // the `new()` method above.
//!     pub fn free(self) -> DmaReg {
//!         Inventory0::free(self.0).0
//!     }
//!
//!     // This method takes `self` by reference and returns a scoped guard object. It
//!     // enables DMA, and the returned guard will automatically disable it on `drop`.
//!     pub fn enable(&mut self) -> inventory::Guard<'_, DmaEn> {
//!         self.setup();
//!         Inventory0::guard(&mut self.0)
//!     }
//!
//!     // This method takes `self` by value and returns the inventory object with one
//!     // token taken. It enables DMA, and in order to disable it, one should
//!     // explicitly call  `from_enabled()` method below.
//!     pub fn into_enabled(self) -> Inventory1<DmaEn> {
//!         self.setup();
//!         let (enabled, token) = self.0.share1();
//!         // To be recreated in `from_enabled()`.
//!         drop(token);
//!         enabled
//!     }
//!
//!     // This method takes the inventory object with one token taken, restores the
//!     // token, and disables DMA.
//!     pub fn from_enabled(enabled: Inventory1<DmaEn>) -> Self {
//!         // Restoring the token dropped in `into_enabled()`.
//!         let token = unsafe { inventory::Token::new() };
//!         let mut enabled = enabled.merge1(token);
//!         Inventory0::teardown(&mut enabled);
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
//!     // It is called only by `Guard::drop` or `Inventory0::teardown`.
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
pub struct Inventory<T: Item, C> {
    item: T,
    count: PhantomData<C>,
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

impl<T: Item> Inventory<T, Count0> {
    /// Creates a new [`Inventory`] in the inactive state with zero tokens
    /// emitted.
    ///
    /// `item` should contain some form of token.
    #[inline]
    pub fn new(item: T) -> Self {
        Self { item, count: PhantomData }
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

#[allow(clippy::unused_self)]
impl<T: Item, C> Inventory<T, C> {
    /// Returns a reference to [`Token`]`<T>`. While the reference exists, the
    /// item is always in its active state.
    #[inline]
    pub fn inventory_token(&self) -> &Token<T> {
        &Token(PhantomData)
    }
}

impl<T: Item, C> Deref for Inventory<T, C> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        &self.item
    }
}

impl<T: Item, C> DerefMut for Inventory<T, C> {
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

macro_rules! define_counters {
    ($($count:ident $alias:ident,)*) => {
        $(
            /// A token counter.
            pub struct $count;

            /// [`Inventory`] with bounded count.
            pub type $alias<T> = Inventory<T, $count>;
        )*
    };
}

macro_rules! define_methods {
    (
        $(
            $subject:ident
            $(($share:ident $share_to:ident $($share_token:ident)*))*
            $([$merge:ident $merge_to:ident $($merge_token:ident)*])*
        )*
    ) => {
        $(
            impl<T: Item> Inventory<T, $subject> {
                $(
                    /// Returns a token and a new inventory object with
                    /// increased counter in its type.
                    pub fn $share(
                        self,
                    ) -> (Inventory<T, $share_to>, $(Token<$share_token>),*) {
                        (
                            Inventory {
                                item: self.item,
                                count: PhantomData,
                            },
                            $(Token(PhantomData::<$share_token>),)*
                        )
                    }
                )*
                $(
                    /// Consumes a token and returns a new inventory object with
                    /// decreased counter in its type.
                    #[allow(clippy::too_many_arguments)]
                    pub fn $merge(
                        self,
                        $($merge_token: Token<T>,)*
                    ) -> Inventory<T, $merge_to> {
                        $(drop($merge_token);)*
                        Inventory {
                            item: self.item,
                            count: PhantomData,
                        }
                    }
                )*
            }
        )*
    };
}

define_counters! {
    Count0 Inventory0,
    Count1 Inventory1,
    Count2 Inventory2,
    Count3 Inventory3,
    Count4 Inventory4,
    Count5 Inventory5,
    Count6 Inventory6,
    Count7 Inventory7,
    Count8 Inventory8,
    Count9 Inventory9,
    Count10 Inventory10,
    Count11 Inventory11,
}

define_methods! {
    Count0
        (share1 Count1 T)
    Count1
        (share1 Count2 T)
        (share2 Count3 T T)
        (share3 Count4 T T T)
        (share4 Count5 T T T T)
        (share5 Count6 T T T T T)
        (share6 Count7 T T T T T T)
        (share7 Count8 T T T T T T T)
        (share8 Count9 T T T T T T T T)
        (share9 Count10 T T T T T T T T T)
        (share10 Count11 T T T T T T T T T T)
        [merge1 Count0 a]
    Count2
        (share1 Count3 T)
        (share2 Count4 T T)
        (share3 Count5 T T T)
        (share4 Count6 T T T T)
        (share5 Count7 T T T T T)
        (share6 Count8 T T T T T T)
        (share7 Count9 T T T T T T T)
        (share8 Count10 T T T T T T T T)
        (share9 Count11 T T T T T T T T T)
        [merge1 Count1 a]
    Count3
        (share1 Count4 T)
        (share2 Count5 T T)
        (share3 Count6 T T T)
        (share4 Count7 T T T T)
        (share5 Count8 T T T T T)
        (share6 Count9 T T T T T T)
        (share7 Count10 T T T T T T T)
        (share8 Count11 T T T T T T T T)
        [merge1 Count2 a]
        [merge2 Count1 a b]
    Count4
        (share1 Count5 T)
        (share2 Count6 T T)
        (share3 Count7 T T T)
        (share4 Count8 T T T T)
        (share5 Count9 T T T T T)
        (share6 Count10 T T T T T T)
        (share7 Count11 T T T T T T T)
        [merge1 Count3 a]
        [merge2 Count2 a b]
        [merge3 Count1 a b c]
    Count5
        (share1 Count6 T)
        (share2 Count7 T T)
        (share3 Count8 T T T)
        (share4 Count9 T T T T)
        (share5 Count10 T T T T T)
        (share6 Count11 T T T T T T)
        [merge1 Count4 a]
        [merge2 Count3 a b]
        [merge3 Count2 a b c]
        [merge4 Count1 a b c d]
    Count6
        (share1 Count7 T)
        (share2 Count8 T T)
        (share3 Count9 T T T)
        (share4 Count10 T T T T)
        (share5 Count11 T T T T T)
        [merge1 Count5 a]
        [merge2 Count4 a b]
        [merge3 Count3 a b c]
        [merge4 Count2 a b c d]
        [merge5 Count1 a b c d e]
    Count7
        (share1 Count8 T)
        (share2 Count9 T T)
        (share3 Count10 T T T)
        (share4 Count11 T T T T)
        [merge1 Count6 a]
        [merge2 Count5 a b]
        [merge3 Count4 a b c]
        [merge4 Count3 a b c d]
        [merge5 Count2 a b c d e]
        [merge6 Count1 a b c d e f]
    Count8
        (share1 Count9 T)
        (share2 Count10 T T)
        (share3 Count11 T T T)
        [merge1 Count7 a]
        [merge2 Count6 a b]
        [merge3 Count5 a b c]
        [merge4 Count4 a b c d]
        [merge5 Count3 a b c d e]
        [merge6 Count2 a b c d e f]
        [merge7 Count1 a b c d e f g]
    Count9
        (share1 Count10 T)
        (share2 Count11 T T)
        [merge1 Count8 a]
        [merge2 Count7 a b]
        [merge3 Count6 a b c]
        [merge4 Count5 a b c d]
        [merge5 Count4 a b c d e]
        [merge6 Count3 a b c d e f]
        [merge7 Count2 a b c d e f g]
        [merge8 Count1 a b c d e f g h]
    Count10
        (share1 Count11 T)
        [merge1 Count9 a]
        [merge2 Count8 a b]
        [merge3 Count7 a b c]
        [merge4 Count6 a b c d]
        [merge5 Count5 a b c d e]
        [merge6 Count4 a b c d e f]
        [merge7 Count3 a b c d e f g]
        [merge8 Count2 a b c d e f g h]
        [merge9 Count1 a b c d e f g h i]
    Count11
        [merge1 Count10 a]
        [merge2 Count9 a b]
        [merge3 Count8 a b c]
        [merge4 Count7 a b c d]
        [merge5 Count6 a b c d e]
        [merge6 Count5 a b c d e f]
        [merge7 Count4 a b c d e f g]
        [merge8 Count3 a b c d e f g h]
        [merge9 Count2 a b c d e f g h i]
        [merge10 Count1 a b c d e f g h i j]
}
