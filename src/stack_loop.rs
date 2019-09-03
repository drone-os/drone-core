//! This module provides interface to wrap a stackful synchronous code into an
//! asynchronous command loop.
//!
//! **NOTE** A Drone platform crate may re-export this module with its own
//! additions under the same name, in which case it should be used instead.

use crate::{
    fib::{Fiber, FiberState},
    future::fallback::*,
};
use core::{future::Future, pin::Pin};

type SessFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// The trait for declaring a synchronous command loop.
///
/// This trait uses only associated items, thus it doesn't require the type to
/// ever be instantiated.
pub trait StackLoop: Send + 'static {
    /// Token type that allows suspending the task while waiting for a request
    /// result.
    type Context: Context<Self::Req, Self::ReqRes>;

    /// `enum` of all possible commands.
    type Cmd: Send + 'static;

    /// `union` of all possible command results.
    type CmdRes: Send + 'static;

    /// `enum` of all possible requests.
    type Req: Send + 'static;

    /// `union` of all possible request results.
    type ReqRes: Send + 'static;

    /// The stack size in bytes.
    const STACK_SIZE: usize;

    /// The commands runner.
    ///
    /// See [`StackLoop`] for examples.
    fn run_cmd(cmd: Self::Cmd, context: Self::Context) -> Self::CmdRes;

    /// Runs on stack creation.
    #[inline]
    fn on_create() {}

    /// Runs inside the synchronous context before the command loop.
    #[inline]
    fn on_enter() {}

    /// Runs on stack destruction.
    #[inline]
    fn on_drop() {}
}

/// A session type for the synchronous command loop [`StackLoop`].
///
/// A type that implements this trait should wrap the fiber for the command
/// loop.
pub trait Sess: Send {
    /// The command loop interface.
    type StackLoop: StackLoop;

    /// Fiber that runs the command loop.
    type Fiber: Fiber<
            Input = In<<Self::StackLoop as StackLoop>::Cmd, <Self::StackLoop as StackLoop>::ReqRes>,
            Yield = Out<
                <Self::StackLoop as StackLoop>::Req,
                <Self::StackLoop as StackLoop>::CmdRes,
            >,
            Return = !,
        > + Send;

    /// Request error type.
    type Error: Send;

    /// Returns a pinned mutable reference to the fiber.
    fn fib(&mut self) -> Pin<&mut Self::Fiber>;

    /// Returns a future that will return a result for the request `req`.
    fn run_req(
        &mut self,
        req: <Self::StackLoop as StackLoop>::Req,
    ) -> SessFuture<'_, Result<<Self::StackLoop as StackLoop>::ReqRes, Self::Error>>;

    /// Returns a future that will return a result for the command `cmd`.
    fn cmd(
        &mut self,
        cmd: <Self::StackLoop as StackLoop>::Cmd,
    ) -> SessFuture<'_, Result<<Self::StackLoop as StackLoop>::CmdRes, Self::Error>> {
        let mut input = In { cmd };
        Box::pin(asyn(move || {
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

/// A token that allows suspending synchronous code.
pub trait Context<Req, ReqRes>: Copy + 'static {
    /// Creates a new token.
    ///
    /// # Safety
    ///
    /// It is unsafe to create a token inside an inappropriate context.
    unsafe fn new() -> Self;

    /// Makes a new request `req`.
    ///
    /// This method suspends execution of the current task allowing to escape
    /// from synchronous code.
    fn req(self, req: Req) -> ReqRes;
}

/// [`Sess::Fiber`] input.
///
/// See also [`Out`].
#[allow(unions_with_drop_fields)]
pub union In<Cmd, ReqRes> {
    /// Command to run by the command loop.
    cmd: Cmd,
    /// Result for the last request.
    req_res: ReqRes,
}

/// [`Sess::Fiber`] output.
///
/// See also [`In`].
pub enum Out<Req, CmdRes> {
    /// Request that the command loop is waiting for.
    Req(Req),
    /// Result for the last command.
    CmdRes(CmdRes),
}

impl<Cmd, ReqRes> In<Cmd, ReqRes> {
    /// Interprets the input as a command.
    ///
    /// # Safety
    ///
    /// Whether the input is really a command object is unchecked.
    pub unsafe fn into_cmd(self) -> Cmd {
        self.cmd
    }

    /// Interprets the input as a request result.
    ///
    /// # Safety
    ///
    /// Whether the input is really a request result object is unchecked.
    pub unsafe fn into_req_res(self) -> ReqRes {
        self.req_res
    }
}
