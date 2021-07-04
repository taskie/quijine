mod atom;
mod class;
mod context;
mod context_ext;
mod convert;
mod error;
mod flags;
mod module;
mod result;
mod runtime;
mod string;
mod types;
mod util;
mod value;

#[cfg(feature = "c_function_list")]
mod arena;
#[cfg(feature = "c_function_list")]
mod function;

#[macro_use]
pub mod macros;
#[doc(hidden)]
pub mod internal;

pub use quijine_core::raw;

pub use atom::{Atom, PropertyEnum};
pub use class::{Class, ClassProperties};
pub use context::{Context, ContextScope};
pub use context_ext::ContextAddIntrinsicExt;
pub use convert::{FromQj, FromQjMulti, IntoQj, IntoQjAtom, IntoQjMulti};
pub use error::{Error, ErrorKind, ErrorValue, ExternalError};
pub use flags::{EvalFlags, GpnFlags, PropFlags, ReadObjFlags, WriteObjFlags};
pub use module::ModuleDef;
pub use result::{ExternalResult, Result};
pub use runtime::{Runtime, RuntimeScope};
pub use types::{
    BigDecimal, BigFloat, BigInt, Bool, CatchOffset, ClassObject, Exception, Float64, FunctionBytecode, Int, Module,
    Null, Object, String, Symbol, Undefined, Uninitialized, Variant,
};
pub use value::Value;

#[cfg(feature = "c_function_list")]
pub use arena::{CStringArena, DefArena};
#[cfg(feature = "c_function_list")]
pub use function::{CFunctionListBuilder, CFunctionListEntry};

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
