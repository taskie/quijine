mod atom;
mod class;
mod context;
mod convert;
mod data;
mod error;
mod runtime;
mod string;
mod types;
mod util;

#[macro_use]
pub mod macros;

pub use qjncore::{EvalFlags, GPNFlags};

pub use atom::{Atom, PropertyEnum};
pub use class::{Class, ClassMethods};
pub use context::{Context, ContextScope};
pub use convert::{AsData, FromQj, FromQjMulti, IntoQj, IntoQjAtom, IntoQjMulti};
pub use data::Data;
pub use error::{Error, ErrorKind, ErrorValue, ExternalError, ExternalResult, Result};
pub use runtime::{Runtime, RuntimeScope};
pub use types::{
    BigDecimal, BigFloat, BigInt, Bool, CatchOffset, Exception, Float64, FunctionBytecode, Int, Module, Null, Object,
    String, Symbol, Undefined, Uninitialized, Variant,
};

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
