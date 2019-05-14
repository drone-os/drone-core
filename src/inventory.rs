//! A pattern that helps representing stateful resources.

use core::{
  marker::PhantomData,
  ops::{Deref, DerefMut},
};

/// A transparent wrapper that holds the actual resource `T` and is responsible
/// for accounting of tokens.
#[repr(transparent)]
pub struct Inventory<T: InventoryResource, C> {
  value: T,
  count: PhantomData<C>,
}

/// An RAII scoped guard for the resouce `T`.
#[must_use]
pub struct InventoryGuard<'a, T: InventoryResource> {
  borrow: &'a mut T,
  inventory_token: InventoryToken<T>,
}

/// A zero-sized token. While it exists, the resource `T` can't change its
/// state.
pub struct InventoryToken<T: InventoryResource>(PhantomData<T>);

/// An inventory resource interface.
pub trait InventoryResource {
  /// Changes the resource state. Called on [`InventoryGuard`] drop.
  fn teardown(&mut self);
}

impl<T: InventoryResource> Inventory<T, Count0> {
  /// Creates a new [`Inventory`] with zero count.
  ///
  /// `value` must be a singleton.
  #[inline]
  pub fn new(value: T) -> Self {
    Self {
      value,
      count: PhantomData,
    }
  }

  /// Returns the resource back.
  #[inline]
  pub fn free(self) -> T {
    self.value
  }
}

impl<T: InventoryResource, C> Inventory<T, C> {
  /// Creates an RAII scoped guard.
  ///
  /// The resource should change its state before calling this method. The state
  /// will be changed back by [`InventoryResource::teardown`] on the guard drop.
  #[inline]
  pub fn guard(&mut self) -> InventoryGuard<'_, T> {
    InventoryGuard {
      borrow: &mut self.value,
      inventory_token: InventoryToken(PhantomData),
    }
  }
}

impl<T: InventoryResource> InventoryToken<T> {
  /// Creates a new [`InventoryToken`].
  ///
  /// # Safety
  ///
  /// Having more inventory tokens than needed could break the inventory
  /// contract.
  #[inline]
  pub unsafe fn new() -> Self {
    Self(PhantomData)
  }
}

impl<T: InventoryResource, C> Deref for Inventory<T, C> {
  type Target = T;

  #[inline]
  fn deref(&self) -> &T {
    &self.value
  }
}

impl<T: InventoryResource, C> DerefMut for Inventory<T, C> {
  #[inline]
  fn deref_mut(&mut self) -> &mut T {
    &mut self.value
  }
}

impl<'a, T: InventoryResource> InventoryGuard<'a, T> {
  /// Returns a reference to a zero-sized token. While this token exists, the
  /// resource can't change its state.
  #[inline]
  pub fn inventory_token(&self) -> &InventoryToken<T> {
    &self.inventory_token
  }
}

impl<'a, T: InventoryResource> Deref for InventoryGuard<'a, T> {
  type Target = T;

  #[inline]
  fn deref(&self) -> &T {
    self.borrow
  }
}

impl<'a, T: InventoryResource> DerefMut for InventoryGuard<'a, T> {
  #[inline]
  fn deref_mut(&mut self) -> &mut T {
    self.borrow
  }
}

impl<'a, T: InventoryResource> Drop for InventoryGuard<'a, T> {
  #[inline]
  fn drop(&mut self) {
    self.borrow.teardown();
  }
}

macro_rules! define_counters {
  ($($count:ident $alias:ident,)*) => {
    $(
      /// A counter for [`Inventory`].
      pub struct $count;

      /// [`Inventory`] with concrete count.
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
      impl<T: InventoryResource> Inventory<T, $subject> {
        $(
          /// Increases the inventory counter and emits inventory tokens.
          pub fn $share(
            self,
          ) -> (Inventory<T, $share_to>, $(InventoryToken<$share_token>),*) {
            (
              Inventory {
                value: self.value,
                count: PhantomData,
              },
              $(InventoryToken(PhantomData::<$share_token>),)*
            )
          }
        )*
        $(
          /// Decreases the inventory counter and takes inventory tokens.
          #[allow(clippy::too_many_arguments)]
          pub fn $merge(
            self,
            $($merge_token: InventoryToken<T>,)*
          ) -> Inventory<T, $merge_to> {
            $(drop($merge_token);)*
            Inventory {
              value: self.value,
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
}

define_methods! {
  Count0
    (share1 Count1 T)
    (share2 Count2 T T)
    (share3 Count3 T T T)
    (share4 Count4 T T T T)
    (share5 Count5 T T T T T)
    (share6 Count6 T T T T T T)
    (share7 Count7 T T T T T T T)
    (share8 Count8 T T T T T T T T)
  Count1
    (share1 Count2 T)
    (share2 Count3 T T)
    (share3 Count4 T T T)
    (share4 Count5 T T T T)
    (share5 Count6 T T T T T)
    (share6 Count7 T T T T T T)
    (share7 Count8 T T T T T T T)
    [merge1 Count0 a]
  Count2
    (share1 Count3 T)
    (share2 Count4 T T)
    (share3 Count5 T T T)
    (share4 Count6 T T T T)
    (share5 Count7 T T T T T)
    (share6 Count8 T T T T T T)
    [merge1 Count1 a]
    [merge2 Count0 a b]
  Count3
    (share1 Count4 T)
    (share2 Count5 T T)
    (share3 Count6 T T T)
    (share4 Count7 T T T T)
    (share5 Count8 T T T T T)
    [merge1 Count2 a]
    [merge2 Count1 a b]
    [merge3 Count0 a b c]
  Count4
    (share1 Count5 T)
    (share2 Count6 T T)
    (share3 Count7 T T T)
    (share4 Count8 T T T T)
    [merge1 Count3 a]
    [merge2 Count2 a b]
    [merge3 Count1 a b c]
    [merge4 Count0 a b c d]
  Count5
    (share1 Count6 T)
    (share2 Count7 T T)
    (share3 Count8 T T T)
    [merge1 Count4 a]
    [merge2 Count3 a b]
    [merge3 Count2 a b c]
    [merge4 Count1 a b c d]
    [merge5 Count0 a b c d e]
  Count6
    (share1 Count7 T)
    (share2 Count8 T T)
    [merge1 Count5 a]
    [merge2 Count4 a b]
    [merge3 Count3 a b c]
    [merge4 Count2 a b c d]
    [merge5 Count1 a b c d e]
    [merge6 Count0 a b c d e f]
  Count7
    (share1 Count8 T)
    [merge1 Count6 a]
    [merge2 Count5 a b]
    [merge3 Count4 a b c]
    [merge4 Count3 a b c d]
    [merge5 Count2 a b c d e]
    [merge6 Count1 a b c d e f]
    [merge7 Count0 a b c d e f g]
  Count8
    [merge1 Count7 a]
    [merge2 Count6 a b]
    [merge3 Count5 a b c]
    [merge4 Count4 a b c d]
    [merge5 Count3 a b c d e]
    [merge6 Count2 a b c d e f]
    [merge7 Count1 a b c d e f g]
    [merge8 Count0 a b c d e f g h]
}

#[cfg(test)]
mod tests {
  use super::*;
  use alloc::rc::Rc;
  use core::{cell::Cell, mem::forget};

  struct Foo(Inventory0<FooEn>);

  struct FooEn(Rc<Cell<bool>>);

  impl Foo {
    fn new(data: Rc<Cell<bool>>) -> Self {
      Self(Inventory0::new(FooEn(data)))
    }

    fn enable(&mut self) -> InventoryGuard<'_, FooEn> {
      self.setup();
      self.0.guard()
    }

    fn into_enabled(mut self) -> Inventory0<FooEn> {
      self.setup();
      self.0
    }

    fn from_enabled(mut enabled: Inventory0<FooEn>) -> Self {
      enabled.teardown();
      Self(enabled)
    }

    fn setup(&mut self) {
      if (self.0).0.get() {
        panic!("Wasn't disabled");
      }
      (self.0).0.set(true);
    }
  }

  impl InventoryResource for FooEn {
    fn teardown(&mut self) {
      self.0.set(false);
    }
  }

  #[test]
  fn test_borrowed() {
    let counter = Rc::new(Cell::new(false));
    let mut foo = Foo::new(Rc::clone(&counter));
    let guard = foo.enable();
    assert!(counter.get());
    drop(guard);
    assert!(!counter.get());
  }

  #[test]
  fn test_owned() {
    let counter = Rc::new(Cell::new(false));
    let foo = Foo::new(Rc::clone(&counter));
    let foo = foo.into_enabled();
    assert!(counter.get());
    Foo::from_enabled(foo);
    assert!(!counter.get());
  }

  #[test]
  #[should_panic]
  fn test_dropped_guard_borrowed() {
    let counter = Rc::new(Cell::new(false));
    let mut foo = Foo::new(Rc::clone(&counter));
    let guard = foo.enable();
    forget(guard);
    let _ = foo.enable();
  }

  #[test]
  #[should_panic]
  fn test_dropped_guard_owned() {
    let counter = Rc::new(Cell::new(false));
    let mut foo = Foo::new(Rc::clone(&counter));
    let guard = foo.enable();
    forget(guard);
    foo.into_enabled();
  }
}
