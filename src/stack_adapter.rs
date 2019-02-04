//! Adapter pattern for transforming stackful synchronous subroutines into
//! fibers.

use core::{future::Future, marker::Unpin, pin::Pin, sync::atomic::AtomicBool};

/// Interface for running synchronous code asynchronously.
pub trait Adapter: Sized + Send + 'static {
  /// See (`Stack`)[Stack].
  type Stack: Stack<Self::Cmd, Self::CmdRes, Self::Req, Self::ReqRes>;

  /// See (`Context`)[Context].
  type Context: Context<Self::Req, Self::ReqRes>;

  /// `enum` of all possible commands.
  type Cmd: Send + 'static;

  /// `union` of all possible command results.
  type CmdRes: Send + 'static;

  /// `enum` of all possible requests.
  type Req: Send + 'static;

  /// `union` of all possible request results.
  type ReqRes: Send + 'static;

  /// Session error.
  type Error: Send;

  /// Stack size.
  const STACK_SIZE: usize;

  /// Returns a mutable reference to the stack fiber.
  fn stack(&mut self) -> &mut Self::Stack;

  /// Runs a command `cmd` synchronously.
  fn run_cmd(cmd: Self::Cmd, context: Self::Context) -> Self::CmdRes;

  /// Runs a request `req` asynchronously.
  #[allow(clippy::type_complexity)]
  fn run_req<'a>(
    &'a mut self,
    req: Self::Req,
  ) -> Pin<
    Box<dyn Future<Output = Result<Self::ReqRes, Self::Error>> + Send + 'a>,
  >;

  /// Controls whether single instance of the code should be enforced.
  ///
  /// If the code is not reentrant, the method should return a reference to a
  /// static atomic boolean.
  fn singleton() -> Option<&'static AtomicBool> {
    None
  }

  /// Runs at the first entrance into command loop.
  fn init() {}

  /// Runs on destruction.
  fn deinit() {}

  /// Returns a future that runs a command `cmd`, and returns its result.
  #[allow(clippy::type_complexity)]
  fn cmd<'a>(
    &'a mut self,
    cmd: Self::Cmd,
  ) -> Pin<
    Box<dyn Future<Output = Result<Self::CmdRes, Self::Error>> + Send + 'a>,
  > {
    let mut input = In { cmd };
    Box::pin(asnc(move || loop {
      input = match self.stack().resume(input) {
        Out::Req(req) => In {
          req_res: awt!(self.run_req(req))?,
        },
        Out::CmdRes(res) => break Ok(res),
      }
    }))
  }
}

/// Stackful fiber that runs synchronous code.
///
/// # Safety
///
/// Implementation must respect the following configuration methods:
///
/// * [`Adapter::singleton`](Adapter::singleton)
/// * [`Adapter::init`](Adapter::init)
/// * [`Adapter::deinit`](Adapter::deinit)
pub unsafe trait Stack<Cmd, CmdRes, Req, ReqRes>:
  Unpin + Sized + Send + 'static
{
  /// Resumes the execution of this fiber.
  fn resume(&mut self, input: In<Cmd, ReqRes>) -> Out<Req, CmdRes>;
}

/// A handler type to make requests.
pub trait Context<Req, ReqRes>: Sized + 'static {
  /// Creates a new `Context`.
  ///
  /// # Safety
  ///
  /// Should be used only inside the code that runs inside the adapter.
  unsafe fn new() -> Self;

  /// Makes a request.
  fn req(&self, req: Req) -> ReqRes;
}

/// Adapter input message.
#[allow(unions_with_drop_fields)]
pub union In<Cmd, ReqRes> {
  /// Command.
  cmd: Cmd,
  /// Request result.
  req_res: ReqRes,
}

/// Adapter output message.
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
