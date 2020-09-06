#[macro_use]
extern crate bitflags;

#[macro_use]
mod core;

mod aliases;
mod class;
mod class_util;
mod context;
mod conversion;
mod error;
mod instance;
mod runtime;
mod string;

pub mod tags;

pub use crate::core::EvalFlags as QjEvalFlags;

pub use aliases::*;
pub use context::{QjContext, QjContextGuard};
pub use error::{QjError, QjErrorValue, QjResult};
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
