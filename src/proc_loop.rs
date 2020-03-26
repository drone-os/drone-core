//! This module provides interface to wrap a stackful synchronous code into an
//! asynchronous command loop.
//!
//! **NOTE** A Drone platform crate may re-export this module with its own
//! additions under the same name, in which case it should be used instead.

#![allow(clippy::wildcard_imports)]

use crate::{
    fib::{self, Fiber},
    future::fallback::*,
};
use core::{future::Future, mem::ManuallyDrop, pin::Pin};

type SessFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// The trait for declaring a synchronous command loop.
///
/// This trait uses only associated items, thus it doesn't require the type to
/// ever be instantiated.
pub trait ProcLoop: Send + 'static {
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

    /// Size of the process stack in bytes.
    const STACK_SIZE: usize;

    /// The commands runner.
    ///
    /// See [`ProcLoop`] for examples.
    fn run_cmd(cmd: Self::Cmd, context: Self::Context) -> Self::CmdRes;

    /// Runs on the process creation.
    #[inline]
    fn on_create() {}

    /// Runs inside the synchronous context before the command loop.
    #[inline]
    fn on_enter() {}

    /// Runs on the process destruction.
    #[inline]
    fn on_drop() {}
}

/// A session type for the synchronous command loop [`ProcLoop`].
///
/// A type that implements this trait should wrap the fiber for the command
/// loop.
pub trait Sess: Send {
    /// The command loop interface.
    type ProcLoop: ProcLoop;

    /// Fiber that runs the command loop.
    type Fiber: Fiber<
            Input = In<<Self::ProcLoop as ProcLoop>::Cmd, <Self::ProcLoop as ProcLoop>::ReqRes>,
            Yield = Out<<Self::ProcLoop as ProcLoop>::Req, <Self::ProcLoop as ProcLoop>::CmdRes>,
            Return = !,
        > + Send;

    /// Request error type.
    type Error: Send;

    /// Returns a pinned mutable reference to the fiber.
    fn fib(&mut self) -> Pin<&mut Self::Fiber>;

    /// Returns a future that will return a result for the request `req`.
    fn run_req(
        &mut self,
        req: <Self::ProcLoop as ProcLoop>::Req,
    ) -> SessFuture<'_, Result<<Self::ProcLoop as ProcLoop>::ReqRes, Self::Error>>;

    /// Returns a future that will return a result for the command `cmd`.
    fn cmd(
        &mut self,
        cmd: <Self::ProcLoop as ProcLoop>::Cmd,
    ) -> SessFuture<'_, Result<<Self::ProcLoop as ProcLoop>::CmdRes, Self::Error>> {
        let mut input = In::from_cmd(cmd);
        Box::pin(asyn(move || {
            loop {
                let fib::Yielded(output) = self.fib().resume(input);
                input = match output {
                    Out::Req(req) => In::from_req_res(awt!(self.run_req(req))?),
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
pub union In<Cmd, ReqRes> {
    /// Command to run by the command loop.
    cmd: ManuallyDrop<Cmd>,
    /// Result for the last request.
    req_res: ManuallyDrop<ReqRes>,
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
    /// Creates a new command input.
    pub fn from_cmd(cmd: Cmd) -> Self {
        Self { cmd: ManuallyDrop::new(cmd) }
    }

    /// Creates a new request result input.
    pub fn from_req_res(req_res: ReqRes) -> Self {
        Self { req_res: ManuallyDrop::new(req_res) }
    }

    /// Interprets the input as a command.
    ///
    /// # Safety
    ///
    /// Whether the input is really a command object is unchecked.
    pub unsafe fn into_cmd(self) -> Cmd {
        ManuallyDrop::into_inner(self.cmd)
    }

    /// Interprets the input as a request result.
    ///
    /// # Safety
    ///
    /// Whether the input is really a request result object is unchecked.
    pub unsafe fn into_req_res(self) -> ReqRes {
        ManuallyDrop::into_inner(self.req_res)
    }
}
