mod class;
mod class_util;
mod context;
mod error;
mod instance;
mod runtime;
mod string;

pub mod tags;
pub mod types;

pub use qjncore::EvalFlags;

pub use class::{Class, ClassMethods};
pub use context::{Context, ContextScope};
pub use error::{Error, ErrorKind, ErrorValue, Result};
pub use instance::Data;
pub use runtime::{Runtime, RuntimeScope};

#[inline]
pub fn new_runtime_scope() -> RuntimeScope {
    RuntimeScope::new()
}

#[inline]
pub fn run<F, R>(f: F) -> Result<R>
where
    F: FnOnce(Runtime) -> Result<R>,
{
    let rts = new_runtime_scope();
    rts.run(f)
}

#[inline]
pub fn run_with_context<F, R>(f: F) -> Result<R>
where
    F: FnOnce(Context) -> Result<R>,
{
    let rts = new_runtime_scope();
    rts.run_with_context(f)
}
