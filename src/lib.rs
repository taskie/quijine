extern crate bitflags;

mod aliases;
mod class;
mod class_util;
mod context;
mod error;
mod instance;
mod runtime;
mod string;

pub mod tags;

pub use qjncore::EvalFlags as QjEvalFlags;

pub use aliases::*;
pub use class::{QjClass, QjClassMethods};
pub use context::{QjContext, QjContextGuard};
pub use error::{QjError, QjErrorValue, QjResult};
pub use instance::Qj;
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
