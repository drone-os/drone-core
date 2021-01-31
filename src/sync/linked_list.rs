//! A lock-free singly-linked list.

use core::{
    iter::{FromIterator, FusedIterator},
    marker::PhantomData,
    ops::{Deref, DerefMut},
    ptr,
    sync::atomic::{AtomicPtr, Ordering},
};

/// A lock-free singly-linked list.
pub struct LinkedList<T> {
    head: AtomicPtr<Node<T>>,
    marker: PhantomData<Box<Node<T>>>,
}

/// A node of [`LinkedList`].
pub struct Node<T> {
    next: *mut Node<T>,
    value: T,
}

/// An owning iterator over the elements of a [`LinkedList`].
pub struct IntoIter<T> {
    list: LinkedList<T>,
}

/// An iterator produced by [`LinkedList::iter_mut`] or
/// [`LinkedList::iter_mut_unchecked`].
pub struct IterMut<'a, T> {
    curr: *mut Node<T>,
    marker: PhantomData<&'a mut Node<T>>,
}

/// An iterator produced by [`LinkedList::drain_filter`] or
/// [`LinkedList::drain_filter_unchecked`].
pub struct DrainFilter<'a, T, F>
where
    F: FnMut(&mut T) -> bool,
{
    head: &'a AtomicPtr<Node<T>>,
    prev: *mut Node<T>,
    curr: *mut Node<T>,
    filter: F,
}

unsafe impl<T> Sync for LinkedList<T> {}

impl<T> LinkedList<T> {
    /// Creates an empty [`LinkedList`].
    ///
    /// # Examples
    ///
    /// ```
    /// use drone_core::sync::LinkedList;
    ///
    /// let list: LinkedList<u32> = LinkedList::new();
    /// ```
    #[inline]
    pub const fn new() -> Self {
        Self { head: AtomicPtr::new(ptr::null_mut()), marker: PhantomData }
    }

    /// Returns `true` if the [`LinkedList`] is empty.
    ///
    /// This operation should compute in *O*(1) time.
    ///
    /// # Examples
    ///
    /// ```
    /// use drone_core::sync::LinkedList;
    ///
    /// let list = LinkedList::new();
    /// assert!(list.is_empty());
    ///
    /// list.push("foo");
    /// assert!(!list.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.head.load(Ordering::Relaxed).is_null()
    }

    /// Adds an element first in the list.
    ///
    /// This operation should compute in *O*(1) time.
    ///
    /// # Examples
    ///
    /// ```
    /// use drone_core::sync::LinkedList;
    ///
    /// let list = LinkedList::new();
    ///
    /// list.push(2);
    /// list.push(1);
    /// assert_eq!(list.pop().unwrap(), 1);
    /// assert_eq!(list.pop().unwrap(), 2);
    /// ```
    #[inline]
    pub fn push(&self, data: T) {
        self.push_node(Box::new(Node::from(data)));
    }

    /// Variant of [`push`], but with boxed node. The node is guaranteed to not
    /// be relocated until removed from the linked list.
    ///
    /// # Examples
    ///
    /// ```
    /// use drone_core::sync::linked_list::{LinkedList, Node};
    ///
    /// let list = LinkedList::new();
    ///
    /// let foo = Box::new(Node::from("foo"));
    /// let foo_ptr: *const Node<&'static str> = &*foo;
    /// list.push_node(foo);
    /// assert_eq!(unsafe { *(*foo_ptr) }, "foo");
    /// ```
    #[inline]
    pub fn push_node(&self, node: Box<Node<T>>) {
        let node = Box::into_raw(node);
        loop {
            let curr = self.head.load(Ordering::Relaxed);
            unsafe { (*node).next = curr };
            if self
                .head
                .compare_exchange_weak(curr, node, Ordering::Release, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
        }
    }

    /// Removes the first element and returns it, or `None` if the list is
    /// empty.
    ///
    /// This operation should compute in *O*(1) time.
    ///
    /// # Examples
    ///
    /// ```
    /// use drone_core::sync::LinkedList;
    ///
    /// let d = LinkedList::new();
    /// assert_eq!(d.pop(), None);
    ///
    /// d.push(1);
    /// d.push(3);
    /// assert_eq!(d.pop(), Some(3));
    /// assert_eq!(d.pop(), Some(1));
    /// assert_eq!(d.pop(), None);
    /// ```
    #[inline]
    pub fn pop(&self) -> Option<T> {
        self.pop_node().map(Node::into_value)
    }

    /// Variant of [`pop`], but returns a boxed node.
    ///
    /// # Examples
    ///
    /// ```
    /// use drone_core::sync::linked_list::{LinkedList, Node};
    ///
    /// let list = LinkedList::new();
    ///
    /// let foo = Box::new(Node::from("foo"));
    /// let foo_ptr: *const Node<&'static str> = &*foo;
    /// list.push_node(foo);
    /// let foo = list.pop_node().unwrap();
    /// list.push_node(foo);
    ///
    /// assert_eq!(unsafe { *(*foo_ptr) }, "foo");
    /// ```
    #[inline]
    pub fn pop_node(&self) -> Option<Box<Node<T>>> {
        loop {
            let curr = self.head.load(Ordering::Acquire);
            if curr.is_null() {
                break None;
            }
            let next = unsafe { (*curr).next };
            if self
                .head
                .compare_exchange_weak(curr, next, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                break Some(unsafe { Box::from_raw(curr) });
            }
        }
    }

    /// Provides a forward iterator with mutable references.
    ///
    /// # Examples
    ///
    /// ```
    /// use drone_core::sync::LinkedList;
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
    /// let mut iter = list.iter_mut();
    /// assert_eq!(iter.next(), Some(&mut 12));
    /// assert_eq!(iter.next(), Some(&mut 11));
    /// assert_eq!(iter.next(), Some(&mut 10));
    /// assert_eq!(iter.next(), None);
    /// ```
    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        // Because `self` is a unique reference, no node can be deleted.
        unsafe { self.iter_mut_unchecked() }
    }

    /// Unsafe variant of [`iter_mut`] with non-mutable `self`.
    ///
    /// # Safety
    ///
    /// Until the iterator is dropped, no nodes shouldn't be removed.
    #[inline]
    pub unsafe fn iter_mut_unchecked(&self) -> IterMut<'_, T> {
        IterMut { curr: self.head.load(Ordering::Acquire), marker: PhantomData }
    }

    /// Creates an iterator which uses a closure to determine if an element
    /// should be removed.
    ///
    /// If the closure returns `true`, then the element is removed and yielded.
    /// If the closure returns `false`, the element will remain in the list and
    /// will not be yielded by the iterator.
    ///
    /// Note that `drain_filter` lets you mutate every element in the filter
    /// closure, regardless of whether you choose to keep or remove it.
    ///
    /// # Examples
    ///
    /// Splitting a list into evens and odds, reusing the original list:
    ///
    /// ```
    /// use drone_core::sync::LinkedList;
    ///
    /// let mut numbers: LinkedList<u32> = LinkedList::new();
    /// numbers.extend(&[1, 2, 3, 4, 5, 6, 8, 9, 11, 13, 14, 15]);
    ///
    /// let evens = numbers.drain_filter(|x| *x % 2 == 0).collect::<LinkedList<_>>();
    /// let odds = numbers;
    ///
    /// assert_eq!(evens.into_iter().collect::<Vec<_>>(), vec![2, 4, 6, 8, 14]);
    /// assert_eq!(odds.into_iter().collect::<Vec<_>>(), vec![15, 13, 11, 9, 5, 3, 1]);
    /// ```
    #[inline]
    pub fn drain_filter<F>(&mut self, filter: F) -> DrainFilter<'_, T, F>
    where
        F: FnMut(&mut T) -> bool,
    {
        // Because `self` is a unique reference, both safety invariants are
        // upholding.
        unsafe { self.drain_filter_unchecked(filter) }
    }

    /// Unsafe variant of [`drain_filter`] with non-mutable `self`.
    ///
    /// # Safety
    ///
    /// * This method is not re-entrant.
    /// * Until the iterator is dropped, no nodes shouldn't be removed.
    #[inline]
    pub unsafe fn drain_filter_unchecked<F>(&self, filter: F) -> DrainFilter<'_, T, F>
    where
        F: FnMut(&mut T) -> bool,
    {
        DrainFilter {
            head: &self.head,
            prev: ptr::null_mut(),
            curr: self.head.load(Ordering::Acquire),
            filter,
        }
    }
}

impl<T> Drop for LinkedList<T> {
    #[inline]
    fn drop(&mut self) {
        let mut curr = self.head.load(Ordering::Acquire);
        while !curr.is_null() {
            let next = unsafe { (*curr).next };
            drop(unsafe { Box::from_raw(curr) });
            curr = next;
        }
    }
}

impl<T> Extend<T> for LinkedList<T> {
    #[inline]
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for elem in iter {
            self.push(elem);
        }
    }
}

impl<'a, T: 'a + Copy> Extend<&'a T> for LinkedList<T> {
    #[inline]
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
        self.extend(iter.into_iter().cloned());
    }
}

impl<T> FromIterator<T> for LinkedList<T> {
    #[inline]
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut list = Self::new();
        list.extend(iter);
        list
    }
}

impl<T> IntoIterator for LinkedList<T> {
    type IntoIter = IntoIter<T>;
    type Item = T;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter { list: self }
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<T> {
        let curr = self.list.head.load(Ordering::Acquire);
        if curr.is_null() {
            None
        } else {
            let next = unsafe { (*curr).next };
            self.list.head.store(next, Ordering::Relaxed);
            Some(unsafe { Box::from_raw(curr) }.into_value())
        }
    }
}

impl<T> FusedIterator for IntoIter<T> {}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.curr.is_null() {
            None
        } else {
            let curr = self.curr;
            self.curr = unsafe { (*self.curr).next };
            Some(unsafe { &mut (*curr).value })
        }
    }
}

impl<T> FusedIterator for IterMut<'_, T> {}

impl<T, F> DrainFilter<'_, T, F>
where
    F: FnMut(&mut T) -> bool,
{
    /// Returns `true` if the iterator has reached the end of the linked list.
    #[inline]
    pub fn is_end(&self) -> bool {
        self.curr.is_null()
    }

    fn cut_out(&mut self, next: *mut Node<T>) {
        if self.prev.is_null() {
            match self.head.compare_exchange(self.curr, next, Ordering::Relaxed, Ordering::Relaxed)
            {
                Ok(_) => return,
                Err(prev) => {
                    self.prev = prev;
                    while unsafe { (*self.prev).next } != self.curr {
                        self.prev = unsafe { (*self.prev).next };
                    }
                }
            }
        }
        unsafe { (*self.prev).next = next };
    }
}

impl<T, F> Iterator for DrainFilter<'_, T, F>
where
    F: FnMut(&mut T) -> bool,
{
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        while !self.is_end() {
            let next = unsafe { (*self.curr).next };
            if (self.filter)(unsafe { &mut (*self.curr).value }) {
                self.cut_out(next);
                let node = unsafe { Box::from_raw(self.curr) };
                self.curr = next;
                return Some(node.into_value());
            }
            self.prev = self.curr;
            self.curr = next;
        }
        None
    }
}

impl<T, F> Drop for DrainFilter<'_, T, F>
where
    F: FnMut(&mut T) -> bool,
{
    #[inline]
    fn drop(&mut self) {
        self.for_each(drop);
    }
}

impl<T, F> FusedIterator for DrainFilter<'_, T, F> where F: FnMut(&mut T) -> bool {}

impl<T> Node<T> {
    /// Unwraps the value.
    #[allow(clippy::boxed_local)]
    #[inline]
    pub fn into_value(self: Box<Self>) -> T {
        self.value
    }
}

impl<T> From<T> for Node<T> {
    #[inline]
    fn from(value: T) -> Self {
        Self { value, next: ptr::null_mut() }
    }
}

impl<T> Deref for Node<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T> DerefMut for Node<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drain_filter_test() {
        let mut m: LinkedList<u32> = LinkedList::new();
        m.extend(&[1, 2, 3, 4, 5, 6]);
        let deleted = m.drain_filter(|v| *v < 4).collect::<Vec<_>>();

        assert_eq!(deleted, &[3, 2, 1]);
        assert_eq!(m.into_iter().collect::<Vec<_>>(), &[6, 5, 4]);
    }

    #[test]
    fn drain_to_empty_test() {
        let mut m: LinkedList<u32> = LinkedList::new();
        m.extend(&[1, 2, 3, 4, 5, 6]);
        let deleted = m.drain_filter(|_| true).collect::<Vec<_>>();

        assert_eq!(deleted, &[6, 5, 4, 3, 2, 1]);
        assert_eq!(m.into_iter().collect::<Vec<_>>(), &[]);
    }
}
