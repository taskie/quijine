#[macro_use]
extern crate bitflags;

mod core;

mod aliases;
mod context;
mod conversion;
mod err;
mod instance;
mod runtime;
mod string;

pub mod tags;

pub use crate::core::EvalFlags as QjEvalFlags;

pub use aliases::*;
pub use context::{QjContext, QjContextGuard};
pub use instance::{Qj, QjVec};
pub use runtime::{QjRuntime, QjRuntimeGuard};

#[inline]
pub fn new_runtime_guard() -> QjRuntimeGuard {
    QjRuntimeGuard::new()
}

#[inline]
pub fn run<F, R>(f: F) -> R
where
    F: FnOnce(QjRuntime) -> R,
{
    let rtg = new_runtime_guard();
    rtg.run(f)
}

#[inline]
pub fn run_with_context<F, R>(f: F) -> R
where
    F: FnOnce(QjContext) -> R,
{
    let rtg = new_runtime_guard();
    rtg.run_with_context(f)
}
