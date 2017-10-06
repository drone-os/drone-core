//! A lock-free single-linked list with owned nodes.
//!
//! See [`LinkedList`] for more details.
//!
//! [`LinkedList`]: struct.LinkedList.html

use core::cmp::Ordering;
use core::hash::{Hash, Hasher};
use core::iter::{FromIterator, FusedIterator};
use core::marker::PhantomData;
use core::ptr;
use core::sync::atomic::AtomicPtr;
use core::sync::atomic::Ordering::*;
use mem::ManuallyDrop;
use prelude::*;

/// A lock-free single-linked list with owned nodes.
///
/// The `LinkedList` allows pushing and popping elements atomically and in
/// constant time.
pub struct LinkedList<T> {
  ptr: AtomicPtr<Node<T>>,
  marker: PhantomData<Box<Node<T>>>,
}

struct Node<T> {
  element: T,
  next: ManuallyDrop<LinkedList<T>>,
}

/// An iterator over the elements of a `LinkedList`.
///
/// This `struct` is created by the [`iter`] method on [`LinkedList`]. See its
/// documentation for more.
///
/// [`iter`]: struct.LinkedList.html#method.iter
/// [`LinkedList`]: struct.LinkedList.html
#[derive(Clone)]
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

/// An owning iterator over the elements of a `LinkedList`.
///
/// This `struct` is created by the [`into_iter`] method on
/// [`LinkedList`][`LinkedList`] (provided by the `IntoIterator` trait). See its
/// documentation for more.
///
/// [`into_iter`]: struct.LinkedList.html#method.into_iter
/// [`LinkedList`]: struct.LinkedList.html
#[derive(Clone)]
pub struct IntoIter<T> {
  head: LinkedList<T>,
}

/// An iterator produced by calling [`drain_filter`] on [`LinkedList`].
///
/// [`drain_filter`]: struct.LinkedList.html#method.drain_filter
/// [`LinkedList`]: struct.LinkedList.html
pub struct DrainFilter<'a, T: 'a, F>
where
  F: FnMut(&mut T) -> bool,
{
  head: &'a mut LinkedList<T>,
  pred: F,
}

impl<T> Node<T> {
  fn new(element: T) -> Self {
    let next = ManuallyDrop::new(LinkedList::new());
    Self { element, next }
  }

  unsafe fn unbox_element(ptr: *mut Self) -> T {
    Box::from_raw(ptr).element
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
    let node = unsafe { self.first_raw() as *mut Node<T> };
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
    let node = unsafe { self.first_raw() as *mut Node<T> };
    if node.is_null() {
      None
    } else {
      unsafe { Some(&mut (*node).element) }
    }
  }

  /// Provides a raw pointer to the first element, or `None` if the list is
  /// empty.
  #[inline]
  pub unsafe fn first_raw<U>(&self) -> *mut U {
    self.ptr.load(Relaxed) as *mut U
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
  pub unsafe fn push_raw<U>(&self, node: *mut U) {
    let node = node as *mut Node<T>;
    loop {
      let current = self.ptr.load(Relaxed);
      (*node).next = ManuallyDrop::new(LinkedList::from(current));
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
    let ptr = unsafe { self.pop_raw() };
    if ptr.is_null() {
      None
    } else {
      Some(unsafe { Node::unbox_element(ptr) })
    }
  }

  /// Removes the first element and returns the raw pointer to it, or `None` if
  /// the list is empty.
  ///
  /// This operation should compute in O(1) time.
  ///
  /// # Safety
  ///
  /// Caller should free the returned pointer manually.
  pub unsafe fn pop_raw<U>(&self) -> *mut U {
    loop {
      let node = self.ptr.load(Relaxed);
      if node.is_null() {
        break node as *mut U;
      }
      let next = (*node).next.ptr.load(Relaxed);
      if self.ptr.compare_and_swap(node, next, Relaxed) == node {
        break node as *mut U;
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
  pub fn clear(&mut self) {
    *self = Self::new();
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
  ///
  /// If the closure returns true, then the element is removed and yielded.  If
  /// the closure returns false, it will try again, and call the closure on the
  /// next element, seeing if it passes the test.
  ///
  /// # Examples
  ///
  /// Splitting a list into evens and odds, reusing the original allocation:
  ///
  /// ```
  /// use drone::collections::LinkedList;
  ///
  /// let mut numbers: LinkedList<u32> = LinkedList::new();
  ///
  /// numbers.push(15);
  /// numbers.push(14);
  /// numbers.push(13);
  /// numbers.push(11);
  /// numbers.push(9);
  /// numbers.push(8);
  /// numbers.push(6);
  /// numbers.push(5);
  /// numbers.push(4);
  /// numbers.push(3);
  /// numbers.push(2);
  /// numbers.push(1);
  ///
  /// let evens = numbers.drain_filter(|x| *x % 2 == 0).collect::<Vec<_>>();
  /// let odds = numbers.into_iter().collect::<Vec<_>>();
  ///
  /// assert_eq!(evens, vec![2, 4, 6, 8, 14]);
  /// assert_eq!(odds, vec![1, 3, 5, 9, 11, 13, 15]);
  /// ```
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

impl<T> Drop for LinkedList<T> {
  fn drop(&mut self) {
    let mut node = *self.ptr.get_mut();
    while !node.is_null() {
      let mut boxed = unsafe { Box::from_raw(node) };
      node = *boxed.next.ptr.get_mut();
    }
  }
}

impl<T> Default for LinkedList<T> {
  /// Creates an empty `LinkedList<T>`.
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

impl<T> From<*mut Node<T>> for LinkedList<T> {
  fn from(ptr: *mut Node<T>) -> Self {
    Self {
      ptr: AtomicPtr::new(ptr),
      marker: PhantomData,
    }
  }
}

impl<T> FromIterator<T> for LinkedList<T> {
  fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
    let mut list = Self::new();
    list.extend(iter);
    list
  }
}

impl<T> IntoIterator for LinkedList<T> {
  type Item = T;
  type IntoIter = IntoIter<T>;

  #[inline]
  fn into_iter(self) -> IntoIter<T> {
    IntoIter { head: self }
  }
}

impl<'a, T> IntoIterator for &'a LinkedList<T> {
  type Item = &'a T;
  type IntoIter = Iter<'a, T>;

  fn into_iter(self) -> Iter<'a, T> {
    self.iter()
  }
}

impl<'a, T> IntoIterator for &'a mut LinkedList<T> {
  type Item = &'a mut T;
  type IntoIter = IterMut<'a, T>;

  fn into_iter(self) -> IterMut<'a, T> {
    self.iter_mut()
  }
}

impl<T> Extend<T> for LinkedList<T> {
  fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
    for elt in iter {
      self.push(elt);
    }
  }
}

impl<'a, T: 'a + Copy> Extend<&'a T> for LinkedList<T> {
  fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
    self.extend(iter.into_iter().cloned());
  }
}

impl<T: PartialEq> PartialEq for LinkedList<T> {
  fn eq(&self, other: &Self) -> bool {
    self.iter().eq(other)
  }
}

impl<T: Eq> Eq for LinkedList<T> {}

impl<T: PartialOrd> PartialOrd for LinkedList<T> {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    self.iter().partial_cmp(other)
  }
}

impl<T: Ord> Ord for LinkedList<T> {
  #[inline]
  fn cmp(&self, other: &Self) -> Ordering {
    self.iter().cmp(other)
  }
}

impl<T: Clone> Clone for LinkedList<T> {
  fn clone(&self) -> Self {
    self.iter().cloned().collect()
  }
}

impl<T: Hash> Hash for LinkedList<T> {
  fn hash<H: Hasher>(&self, state: &mut H) {
    for elt in self {
      elt.hash(state);
    }
  }
}

unsafe impl<T: Send> Send for LinkedList<T> {}

unsafe impl<T: Sync> Sync for LinkedList<T> {}

unsafe impl<'a, T: Sync> Send for Iter<'a, T> {}

unsafe impl<'a, T: Sync> Sync for Iter<'a, T> {}

unsafe impl<'a, T: Send> Send for IterMut<'a, T> {}

unsafe impl<'a, T: Sync> Sync for IterMut<'a, T> {}

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

impl<'a, T> FusedIterator for Iter<'a, T> {}

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

impl<'a, T> FusedIterator for IterMut<'a, T> {}

impl<T> Iterator for IntoIter<T> {
  type Item = T;

  #[inline]
  fn next(&mut self) -> Option<T> {
    self.head.pop()
  }
}

impl<T> FusedIterator for IntoIter<T> {}

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

impl<'a, T, F> FusedIterator for DrainFilter<'a, T, F>
where
  F: FnMut(&mut T) -> bool,
{
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
