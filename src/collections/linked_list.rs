//! An atomic single-linked list with owned nodes.

use core::marker::PhantomData;
use core::ptr;
use core::sync::atomic::AtomicPtr;
use core::sync::atomic::Ordering::*;
use prelude::*;

/// A linked list handler.
pub struct LinkedList<T> {
  ptr: AtomicPtr<Node<T>>,
  marker: PhantomData<Box<Node<T>>>,
}

/// A linked list node.
pub struct Node<T> {
  element: T,
  next: LinkedList<T>,
}

/// An iterator over the elements of a `LinkedList`.
///
/// This `struct` is created by the [`iter`] method on [`LinkedList`]. See its
/// documentation for more.
///
/// [`iter`]: struct.LinkedList.html#method.iter
/// [`LinkedList`]: struct.LinkedList.html
pub struct Iter<'a, T: 'a> {
  head: &'a LinkedList<T>,
}

/// A mutable iterator over the elements of a `LinkedList`.
///
/// This `struct` is created by the [`iter_mut`] method on [`LinkedList`]. See
/// its documentation for more.
///
/// [`iter_mut`]: struct.LinkedList.html#method.iter_mut
/// [`LinkedList`]: struct.LinkedList.html
pub struct IterMut<'a, T: 'a> {
  head: &'a mut LinkedList<T>,
}

/// An iterator produced by calling `drain_filter` on `LinkedList`.
pub struct DrainFilter<'a, T: 'a, F>
where
  F: FnMut(&mut T) -> bool,
{
  head: &'a mut LinkedList<T>,
  pred: F,
}

impl<T> From<*mut Node<T>> for LinkedList<T> {
  fn from(ptr: *mut Node<T>) -> Self {
    Self {
      ptr: AtomicPtr::new(ptr),
      marker: PhantomData,
    }
  }
}

impl<T> LinkedList<T> {
  /// Creates an empty `LinkedList`.
  ///
  /// # Examples
  ///
  /// ```
  /// use drone::collections::LinkedList;
  ///
  /// let list: LinkedList<u32> = LinkedList::new();
  /// ```
  #[inline]
  pub const fn new() -> Self {
    Self {
      ptr: AtomicPtr::new(ptr::null_mut()),
      marker: PhantomData,
    }
  }

  /// Returns `true` if the `LinkedList` is empty.
  ///
  /// This operation should compute in O(1) time.
  ///
  /// # Examples
  ///
  /// ```
  /// use drone::collections::LinkedList;
  ///
  /// let mut dl = LinkedList::new();
  /// assert!(dl.is_empty());
  ///
  /// dl.push("foo");
  /// assert!(!dl.is_empty());
  /// ```
  #[inline]
  pub fn is_empty(&self) -> bool {
    self.ptr.load(Relaxed).is_null()
  }

  /// Provides a reference to the first element, or `None` if the list is empty.
  ///
  /// # Examples
  ///
  /// ```
  /// use drone::collections::LinkedList;
  ///
  /// let mut dl = LinkedList::new();
  /// assert_eq!(dl.first(), None);
  ///
  /// dl.push(1);
  /// assert_eq!(dl.first(), Some(&1));
  /// ```
  #[inline]
  pub fn first(&self) -> Option<&T> {
    let node = self.ptr.load(Relaxed);
    if node.is_null() {
      None
    } else {
      unsafe { Some(&(*node).element) }
    }
  }

  /// Provides a mutable reference to the first element, or `None` if the list
  /// is empty.
  ///
  /// # Examples
  ///
  /// ```
  /// use drone::collections::LinkedList;
  ///
  /// let mut dl = LinkedList::new();
  /// assert_eq!(dl.first(), None);
  ///
  /// dl.push(1);
  /// assert_eq!(dl.first(), Some(&1));
  ///
  /// match dl.first_mut() {
  ///     None => {},
  ///     Some(x) => *x = 5,
  /// }
  /// assert_eq!(dl.first(), Some(&5));
  /// ```
  #[inline]
  pub fn first_mut(&mut self) -> Option<&mut T> {
    let node = self.ptr.load(Relaxed);
    if node.is_null() {
      None
    } else {
      unsafe { Some(&mut (*node).element) }
    }
  }

  /// Adds an element first in the list.
  ///
  /// This operation should compute in O(1) time.
  ///
  /// # Examples
  ///
  /// ```
  /// use drone::collections::LinkedList;
  ///
  /// let mut dl = LinkedList::new();
  ///
  /// dl.push(2);
  /// assert_eq!(dl.first().unwrap(), &2);
  ///
  /// dl.push(1);
  /// assert_eq!(dl.first().unwrap(), &1);
  /// ```
  pub fn push(&self, element: T) {
    unsafe {
      self.push_raw(Box::into_raw(Box::new(Node::new(element))));
    }
  }

  /// Adds a raw pointer to an element first in the list.
  ///
  /// This operation should compute in O(1) time.
  ///
  /// # Safety
  ///
  /// Caller should pass an ownership of the pointer.
  pub unsafe fn push_raw(&self, node: *mut Node<T>) {
    loop {
      let current = self.ptr.load(Relaxed);
      (*node).next = current.into();
      if self.ptr.compare_and_swap(current, node, Relaxed) == current {
        break;
      }
    }
  }

  /// Removes the first element and returns it, or `None` if the list is empty.
  ///
  /// This operation should compute in O(1) time.
  ///
  /// # Examples
  ///
  /// ```
  /// use drone::collections::LinkedList;
  ///
  /// let mut d = LinkedList::new();
  /// assert_eq!(d.pop(), None);
  ///
  /// d.push(1);
  /// d.push(3);
  /// assert_eq!(d.pop(), Some(3));
  /// assert_eq!(d.pop(), Some(1));
  /// assert_eq!(d.pop(), None);
  /// ```
  pub fn pop(&self) -> Option<T> {
    unsafe { self.pop_raw().map(|ptr| Node::unbox_element(ptr)) }
  }

  /// Removes the first element and returns the raw pointer to it, or `None` if
  /// the list is empty.
  ///
  /// This operation should compute in O(1) time.
  ///
  /// # Safety
  ///
  /// Caller should free the returned pointer manually.
  pub unsafe fn pop_raw(&self) -> Option<*mut Node<T>> {
    loop {
      let node = self.ptr.load(Relaxed);
      if node.is_null() {
        return None;
      }
      let next = (*node).next.ptr.load(Relaxed);
      if self.ptr.compare_and_swap(node, next, Relaxed) == node {
        return Some(node);
      }
    }
  }

  /// Removes all elements from the `LinkedList`.
  ///
  /// This operation should compute in O(n) time.
  ///
  /// # Examples
  ///
  /// ```
  /// use drone::collections::LinkedList;
  ///
  /// let mut dl = LinkedList::new();
  ///
  /// dl.push(2);
  /// dl.push(1);
  /// assert!(!dl.is_empty());
  /// assert_eq!(dl.first(), Some(&1));
  ///
  /// dl.clear();
  /// assert!(dl.is_empty());
  /// assert_eq!(dl.first(), None);
  /// ```
  pub fn clear(&self) {
    loop {
      let mut node = self.ptr.load(Relaxed);
      if node.is_null() {
        break;
      }
      if self.ptr.compare_and_swap(node, ptr::null_mut(), Relaxed) == node {
        while !node.is_null() {
          let boxed = unsafe { Box::from_raw(node) };
          node = boxed.next.ptr.load(Relaxed);
          drop(boxed);
        }
        break;
      }
    }
  }

  /// Returns `true` if the `LinkedList` contains an element equal to the given
  /// value.
  ///
  /// # Examples
  ///
  /// ```
  /// use drone::collections::LinkedList;
  ///
  /// let mut list: LinkedList<u32> = LinkedList::new();
  ///
  /// list.push(0);
  /// list.push(1);
  /// list.push(2);
  ///
  /// assert_eq!(list.contains(&0), true);
  /// assert_eq!(list.contains(&10), false);
  /// ```
  pub fn contains(&self, x: &T) -> bool
  where
    T: PartialEq<T>,
  {
    self.iter().any(|e| e == x)
  }

  /// Provides a forward iterator.
  ///
  /// # Examples
  ///
  /// ```
  /// use drone::collections::LinkedList;
  ///
  /// let mut list: LinkedList<u32> = LinkedList::new();
  ///
  /// list.push(0);
  /// list.push(1);
  /// list.push(2);
  ///
  /// let mut iter = list.iter();
  /// assert_eq!(iter.next(), Some(&2));
  /// assert_eq!(iter.next(), Some(&1));
  /// assert_eq!(iter.next(), Some(&0));
  /// assert_eq!(iter.next(), None);
  /// ```
  #[inline]
  pub fn iter(&self) -> Iter<T> {
    Iter { head: self }
  }

  /// Provides a forward iterator with mutable references.
  ///
  /// # Examples
  ///
  /// ```
  /// use drone::collections::LinkedList;
  ///
  /// let mut list: LinkedList<u32> = LinkedList::new();
  ///
  /// list.push(0);
  /// list.push(1);
  /// list.push(2);
  ///
  /// for element in list.iter_mut() {
  ///     *element += 10;
  /// }
  ///
  /// let mut iter = list.iter();
  /// assert_eq!(iter.next(), Some(&12));
  /// assert_eq!(iter.next(), Some(&11));
  /// assert_eq!(iter.next(), Some(&10));
  /// assert_eq!(iter.next(), None);
  /// ```
  #[inline]
  pub fn iter_mut(&mut self) -> IterMut<T> {
    IterMut { head: self }
  }

  /// Creates an iterator which uses a closure to determine if an element should
  /// be removed.
  #[inline]
  pub fn drain_filter<F>(&mut self, filter: F) -> DrainFilter<T, F>
  where
    F: FnMut(&mut T) -> bool,
  {
    DrainFilter {
      head: self,
      pred: filter,
    }
  }
}

impl<T> Node<T> {
  /// Creates a detached `Node`.
  pub fn new(element: T) -> Self {
    let next = LinkedList::new();
    Self { element, next }
  }

  /// Takes an ownership of the `ptr` memory and returns the inner element.
  ///
  /// # Safety
  ///
  /// `ptr` should be a valid pointer to a node.
  pub unsafe fn unbox_element(ptr: *mut Self) -> T {
    Box::from_raw(ptr).element
  }
}

impl<'a, T> Iterator for Iter<'a, T> {
  type Item = &'a T;

  fn next(&mut self) -> Option<&'a T> {
    let node = self.head.ptr.load(Relaxed);
    if node.is_null() {
      None
    } else {
      unsafe {
        self.head = &(*node).next;
        Some(&(*node).element)
      }
    }
  }
}

impl<'a, T> Iterator for IterMut<'a, T> {
  type Item = &'a mut T;

  fn next(&mut self) -> Option<&'a mut T> {
    let node = self.head.ptr.load(Relaxed);
    if node.is_null() {
      None
    } else {
      unsafe {
        self.head = &mut (*node).next;
        Some(&mut (*node).element)
      }
    }
  }
}

impl<'a, T, F> Iterator for DrainFilter<'a, T, F>
where
  F: FnMut(&mut T) -> bool,
{
  type Item = T;

  fn next(&mut self) -> Option<T> {
    loop {
      let node = self.head.ptr.load(Relaxed);
      if node.is_null() {
        return None;
      }
      unsafe {
        if (self.pred)(&mut (*node).element) {
          let next = (*node).next.ptr.load(Relaxed);
          while self.head.ptr.compare_and_swap(node, next, Relaxed) != node {
            let next = self.head.ptr.load(Relaxed);
            self.head = &mut (*next).next;
          }
          return Some(Node::unbox_element(node));
        } else {
          self.head = &mut (*node).next;
        }
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::mem;

  #[test]
  fn unit_list() {
    assert_eq!(mem::size_of::<Node<()>>(), mem::size_of::<usize>());
  }
}
