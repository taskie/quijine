mod atom;
mod class;
mod context;
mod context_ext;
mod convert;
mod error;
mod runtime;
mod string;
mod types;
mod util;
mod value;

#[macro_use]
pub mod macros;

pub use quijine_core::{EvalFlags, GPNFlags, PropFlags};

pub use atom::{Atom, PropertyEnum};
pub use class::{Class, ClassProperties};
pub use context::{Context, ContextScope};
pub use context_ext::ContextAddIntrinsicExt;
pub use convert::{FromQj, FromQjMulti, IntoQj, IntoQjAtom, IntoQjMulti};
pub use error::{Error, ErrorKind, ErrorValue, ExternalError, ExternalResult, Result};
pub use runtime::{Runtime, RuntimeScope};
pub use types::{
    BigDecimal, BigFloat, BigInt, Bool, CatchOffset, ClassObject, Exception, Float64, FunctionBytecode, Int, Module,
    Null, Object, String, Symbol, Undefined, Uninitialized, Variant,
};
pub use value::Value;

#[inline]
pub fn run<F, R>(f: F) -> Result<R>
where
    F: FnOnce(Runtime) -> Result<R>,
{
    let rts = RuntimeScope::new();
    rts.run(f)
}

#[inline]
pub fn context<F, R>(f: F) -> Result<R>
where
    F: FnOnce(Context) -> Result<R>,
{
    let rts = RuntimeScope::new();
    rts.run_with_context(f)
}
