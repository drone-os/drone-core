//! Machinery for wrapping stackful synchronous code into stackless asynchronous
//! command loop.

use crate::fib::{Fiber, FiberState};
use core::{future::Future, pin::Pin};

/// A type responsive for handling requests from synchronous code.
pub trait StackLoopSess: Send {
  /// Wrapped stack.
  type Stack: Stack;

  /// A fiber for [`StackLoopSess::Stack`].
  type Fiber: Fiber<
      Input = In<<Self::Stack as Stack>::Cmd, <Self::Stack as Stack>::ReqRes>,
      Yield = Out<<Self::Stack as Stack>::Req, <Self::Stack as Stack>::CmdRes>,
      Return = !,
    > + Send;

  /// Session error.
  type Error: Send;

  /// Returns a pinned mutable reference to the wrapped fiber.
  fn fib(&mut self) -> Pin<&mut Self::Fiber>;

  /// Runs a request `req` asynchronously.
  #[allow(clippy::type_complexity)]
  fn run_req<'sess>(
    &'sess mut self,
    req: <Self::Stack as Stack>::Req,
  ) -> Pin<
    Box<
      dyn Future<Output = Result<<Self::Stack as Stack>::ReqRes, Self::Error>>
        + Send
        + 'sess,
    >,
  >;

  /// Returns a future that runs a command `cmd`, and returns its result.
  #[allow(clippy::type_complexity)]
  fn cmd<'sess>(
    &'sess mut self,
    cmd: <Self::Stack as Stack>::Cmd,
  ) -> Pin<
    Box<
      dyn Future<Output = Result<<Self::Stack as Stack>::CmdRes, Self::Error>>
        + Send
        + 'sess,
    >,
  > {
    let mut input = In { cmd };
    Box::pin(asnc(move || {
      loop {
        let FiberState::Yielded(output) = self.fib().resume(input);
        input = match output {
          Out::Req(req) => In {
            req_res: awt!(self.run_req(req))?,
          },
          Out::CmdRes(res) => break Ok(res),
        }
      }
    }))
  }
}

/// A type that wraps some stackful synchronous code.
pub trait Stack: Send + 'static {
  /// Request maker.
  type Context: Context<Self::Req, Self::ReqRes>;

  /// `enum` of all possible commands.
  type Cmd: Send + 'static;

  /// `union` of all possible command results.
  type CmdRes: Send + 'static;

  /// `enum` of all possible requests.
  type Req: Send + 'static;

  /// `union` of all possible request results.
  type ReqRes: Send + 'static;

  /// Stack size.
  const STACK_SIZE: usize;

  /// Runs a command `cmd` synchronously.
  fn run_cmd(cmd: Self::Cmd, context: Self::Context) -> Self::CmdRes;

  /// Runs at the first entrance into command loop.
  #[inline]
  fn begin() {}

  /// Runs on destruction.
  #[inline]
  fn end() {}

  /// Runs before stack creation.
  #[inline]
  fn before_create() {}

  /// Runs after stack destruction.
  #[inline]
  fn after_destroy() {}
}

/// A handler to make requests from synchronous code.
pub trait Context<Req, ReqRes>: 'static {
  /// Creates a new `Context`.
  ///
  /// # Safety
  ///
  /// Should be used only inside the wrapped synchronous code.
  unsafe fn new() -> Self;

  /// Makes a request.
  fn req(&self, req: Req) -> ReqRes;
}

/// Stack input message.
#[allow(unions_with_drop_fields)]
pub union In<Cmd, ReqRes> {
  /// Command.
  cmd: Cmd,
  /// Request result.
  req_res: ReqRes,
}

/// Stack output message.
pub enum Out<Req, CmdRes> {
  /// Request.
  Req(Req),
  /// Command result.
  CmdRes(CmdRes),
}

impl<Cmd, ReqRes> In<Cmd, ReqRes> {
  /// Reads the input as a command.
  pub unsafe fn into_cmd(self) -> Cmd {
    self.cmd
  }

  /// Reads the input as a request result.
  pub unsafe fn into_req_res(self) -> ReqRes {
    self.req_res
  }
}
