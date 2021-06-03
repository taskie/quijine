mod class;
mod context;
mod convert;
mod data;
mod error;
mod runtime;
mod string;
mod types;

#[macro_use]
pub mod macros;

pub use qjncore::EvalFlags;

pub use class::{Class, ClassMethods};
pub use context::{Context, ContextScope};
pub use data::Data;
pub use error::{Error, ErrorKind, ErrorValue, ExternalError, ExternalResult, Result};
pub use runtime::{Runtime, RuntimeScope};
pub use types::{
    BigDecimal, BigFloat, BigInt, Bool, CatchOffset, Exception, Float64, Int, Null, Object, String, Symbol, Undefined,
    Uninitialized, Variant,
};

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
