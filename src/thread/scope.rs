use core::marker::PhantomData;
use fiber::FiberFutureScoped;
use thread::prelude::*;

/// Scoped thread.
pub struct ThreadScope<'scope, T, U, S>
where
  T: ThreadToken<U>,
  U: ThreadTag,
  S: 'scope,
{
  thread: T,
  tag: PhantomData<U>,
  token: ThreadScopeToken<'scope, S>,
}

/// Scoped thread token.
pub struct ThreadScopeToken<'scope, S: 'scope>(PhantomData<&'scope S>);

/// Scoped thread guard.
pub struct ThreadScopeGuard<'scope, S: 'scope, T> {
  /// Scope token.
  pub token: ThreadScopeToken<'scope, S>,
  /// Guarded value.
  pub value: T,
}

impl<'scope, T, U, S> ThreadScope<'scope, T, U, S>
where
  T: ThreadToken<U>,
  U: ThreadTag,
  S: 'scope,
{
  #[inline(always)]
  pub(super) fn new(thread: T) -> Self {
    Self {
      thread,
      tag: PhantomData,
      token: ThreadScopeToken(PhantomData),
    }
  }

  #[inline(always)]
  pub(crate) fn into_parts(self) -> (T, ThreadScopeToken<'scope, S>) {
    (self.thread, self.token)
  }

  /// Adds a new fiber to the stack. Returns a `Future` of the fiber's return
  /// value. This method accepts a generator.
  #[inline(always)]
  pub fn future<G, R, E>(self, g: G) -> FiberFutureScoped<'scope, R, E, S>
  where
    G: Generator<Yield = (), Return = Result<R, E>>,
    G: Send + 'scope,
    R: Send + 'scope,
    E: Send + 'scope,
  {
    FiberFutureScoped::new(self, g)
  }

  /// Adds a new fiber to the stack. Returns a `Future` of the fiber's return
  /// value. This method accepts a closure.
  #[inline(always)]
  pub fn future_fn<F, R, E>(self, f: F) -> FiberFutureScoped<'scope, R, E, S>
  where
    F: FnOnce() -> Result<R, E>,
    F: Send + 'scope,
    R: Send + 'scope,
    E: Send + 'scope,
  {
    FiberFutureScoped::new(self, || {
      if false {
        yield;
      }
      f()
    })
  }
}

impl<'scope, S: 'scope> ThreadScopeToken<'scope, S> {
  /// Wraps the token and `value` into `ThreadScopeGuard` struct.
  #[inline(always)]
  pub fn wrap<T>(self, value: T) -> ThreadScopeGuard<'scope, S, T> {
    ThreadScopeGuard { token: self, value }
  }
}

/// Creates a new scope for scoped fibers.
#[macro_export]
macro_rules! scoped_thread {
  ($thread:expr => $scope:ident $block:block) => {
    {
      struct _UniqueThreadScope;
      #[allow(unused_unsafe)]
      let $scope = unsafe { $thread.scope::<_UniqueThreadScope>() };
      match $block? {
        ThreadScopeGuard {
          token: ThreadScopeToken::<_UniqueThreadScope> { .. },
          value,
        } => value,
      }
    }
  }
}
