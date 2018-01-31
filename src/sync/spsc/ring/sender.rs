use super::{Inner, COMPLETE, INDEX_BITS, INDEX_MASK, RX_LOCK};
use alloc::arc::Arc;
use core::fmt::{self, Debug};
use core::ptr;
use core::sync::atomic::Ordering::*;
use sync::spsc::SpscInner;

/// The sending-half of [`ring::channel`].
///
/// [`ring::channel`]: fn.channel.html
pub struct Sender<T, E> {
  inner: Arc<Inner<T, E>>,
}

/// Error returned from [`Sender::send`].
///
/// [`Sender::send`]: struct.Sender.html#method.send
pub struct SendError<T> {
  /// Value which wasn't sent.
  pub value: T,
  /// The error kind.
  pub kind: SendErrorKind,
}

/// Part of [`SendError`].
///
/// [`SendError`]: struct.SendError.html
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SendErrorKind {
  /// The corresponding [`Receiver`] is dropped.
  ///
  /// [`Receiver`]: struct.Receiver.html
  Canceled,
  /// Buffer overflow.
  Overflow,
}

impl<T, E> Sender<T, E> {
  #[inline(always)]
  pub(super) fn new(inner: Arc<Inner<T, E>>) -> Self {
    Self { inner }
  }

  /// Sends a value across the channel.
  ///
  /// [`Receiver`]: struct.Receiver.html
  #[inline(always)]
  pub fn send(&mut self, value: T) -> Result<(), SendError<T>> {
    self.inner.send(value)
  }

  /// Sends a value across the channel. Overwrites on overflow.
  ///
  /// [`Receiver`]: struct.Receiver.html
  #[inline(always)]
  pub fn send_overwrite(&mut self, value: T) -> Result<(), T> {
    self.inner.send_overwrite(value)
  }

  /// Completes this stream with an error.
  ///
  /// If the value is successfully enqueued, then `Ok(())` is returned. If the
  /// receiving end was dropped before this function was called, then `Err` is
  /// returned with the value provided.
  ///
  /// [`Receiver`]: struct.Receiver.html
  #[inline(always)]
  pub fn send_err(self, err: E) -> Result<(), E> {
    self.inner.send_err(err)
  }

  /// Polls this [`Sender`] half to detect whether the [`Receiver`] this has
  /// paired with has gone away.
  ///
  /// # Panics
  ///
  /// Like `Future::poll`, this function will panic if it's not called from
  /// within the context of a task. In other words, this should only ever be
  /// called from inside another future.
  ///
  /// If you're calling this function from a context that does not have a task,
  /// then you can use the [`is_canceled`] API instead.
  ///
  /// [`Sender`]: struct.Sender.html
  /// [`Receiver`]: struct.Receiver.html
  /// [`is_canceled`]: struct.Receiver.html#method.is_canceled
  #[inline(always)]
  pub fn poll_cancel(&mut self) -> Poll<(), ()> {
    self.inner.poll_cancel()
  }

  /// Tests to see whether this [`Sender`]'s corresponding [`Receiver`] has gone
  /// away.
  ///
  /// [`Sender`]: struct.Sender.html
  /// [`Receiver`]: struct.Receiver.html
  #[inline(always)]
  pub fn is_canceled(&self) -> bool {
    self.inner.is_canceled()
  }
}

impl<T, E> Drop for Sender<T, E> {
  #[inline(always)]
  fn drop(&mut self) {
    self.inner.drop_tx();
  }
}

impl<T, E> Inner<T, E> {
  fn send(&self, value: T) -> Result<(), SendError<T>> {
    let state = self.state_load(Relaxed);
    if state & COMPLETE != 0 {
      Err(SendError::new(value, SendErrorKind::Canceled))
    } else if let Some(index) = Self::put_index(state, self.buffer.cap()) {
      self.put(value, state, index)
    } else {
      Err(SendError::new(value, SendErrorKind::Overflow))
    }
  }

  fn send_overwrite(&self, value: T) -> Result<(), T> {
    let mut state = self.state_load(Relaxed);
    loop {
      if state & COMPLETE != 0 {
        break Err(value);
      }
      match Self::put_index(state, self.buffer.cap()) {
        Some(index) => break self.put(value, state, index),
        None => {
          state = self
            .update(state, Relaxed, Relaxed, |state| {
              if let Some(index) = Self::take_index(state, self.buffer.cap()) {
                Ok((*state, index))
              } else {
                Err(*state)
              }
            })
            .map(|(state, index)| {
              unsafe {
                ptr::drop_in_place(self.buffer.ptr().offset(index as isize));
              }
              state
            })
            .unwrap_or_else(|state| state);
        }
      }
    }
  }

  fn send_err(&self, err: E) -> Result<(), E> {
    if self.is_canceled() {
      Err(err)
    } else {
      unsafe { *self.err.get() = Some(err) };
      Ok(())
    }
  }

  #[inline(always)]
  fn put_index(state: usize, capacity: usize) -> Option<usize> {
    let count = state & INDEX_MASK;
    if count == capacity {
      None
    } else {
      let begin = state >> INDEX_BITS & INDEX_MASK;
      let index = begin.wrapping_add(count).wrapping_rem(capacity);
      Some(index)
    }
  }

  #[inline(always)]
  fn put<U>(&self, value: T, state: usize, index: usize) -> Result<(), U> {
    unsafe {
      ptr::write(self.buffer.ptr().offset(index as isize), value);
    }
    self
      .update(state, AcqRel, Relaxed, |state| {
        *state = state.wrapping_add(1);
        if *state & RX_LOCK == 0 {
          *state |= RX_LOCK;
          Ok(Some(*state))
        } else {
          Ok(None)
        }
      })
      .map(|state| {
        state.map(|state| {
          unsafe {
            (*self.rx_task.get()).as_ref().map(|task| task.notify());
          }
          self.update(state, Release, Relaxed, |state| {
            *state ^= RX_LOCK;
            Ok::<(), ()>(())
          })
        });
      })
  }
}

impl<T> SendError<T> {
  #[inline(always)]
  fn new(value: T, kind: SendErrorKind) -> Self {
    SendError { value, kind }
  }
}

impl<T> Debug for SendError<T> {
  #[inline(always)]
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    self.kind.fmt(f)
  }
}

impl<T> PartialEq for SendError<T> {
  #[inline(always)]
  fn eq(&self, other: &Self) -> bool {
    self.kind.eq(&other.kind)
  }
}

impl<T> Eq for SendError<T> {}
