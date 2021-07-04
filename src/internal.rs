use crate::{raw, Context, ModuleDef, Value};
use qc::AsJsValue;
use quijine_core as qc;
use std::os::raw::c_int;

/// This function is used by js_c_function macro.
/// # Safety
/// * A context and values must have valid lifetime.
/// * The length of argv must equal to argc.
#[inline]
pub unsafe fn convert_function_arguments<'q>(
    ctx: *mut raw::JSContext,
    js_this: raw::JSValue,
    argc: c_int,
    argv: *mut raw::JSValue,
) -> (Context<'q>, Value<'q>, Vec<Value<'q>>) {
    let (rctx, rthis, rargs) = qc::convert_function_arguments(ctx, js_this, argc, argv);
    let ctx = Context::from_raw(rctx);
    let this = Value::from_raw_parts(rthis, rctx);
    Value::dup(&this);
    let args: Vec<Value> = rargs.iter().map(|v| Value::from_raw_parts(*v, rctx)).collect();
    args.iter().for_each(Value::dup);
    (ctx, this, args)
}

#[inline]
pub fn convert_function_result(res: Value) -> raw::JSValue {
    Value::dup(&res);
    res.as_raw().as_js_value()
}

/// # Safety
/// * A context must have valid lifetime.
#[doc(hidden)]
#[inline]
pub unsafe fn convert_context<'q>(ctx: *mut raw::JSContext) -> Context<'q> {
    Context::from_raw(qc::Context::from_raw(ctx))
}

/// # Safety
/// * A value and a context must have valid lifetime.
#[doc(hidden)]
#[inline]
pub unsafe fn convert_value_and_dup(val: raw::JSValue, ctx: Context) -> Value {
    let val = Value::from_raw_parts(qc::Value::from_raw(val, ctx.as_raw()), ctx.as_raw());
    Value::dup(&val);
    val
}

/// # Safety
/// * A module def and a context must have valid lifetime.
#[doc(hidden)]
#[inline]
pub unsafe fn convert_module_def(m: *mut raw::JSModuleDef, ctx: Context) -> ModuleDef {
    ModuleDef::from_raw_parts(qc::ModuleDef::from_raw(m, ctx.as_raw()), ctx.as_raw())
}
