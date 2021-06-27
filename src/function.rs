use crate::{context::Context, value::Value};
use quijine_core::{
    self as qc,
    raw::{JSContext, JSValue},
    AsJsValue,
};
use std::os::raw::c_int;

/// This function is used by js_c_function macro.
/// # Safety
/// * A context and values must have valid lifetime.
/// * The length of argv must equal to argc.
#[inline]
pub unsafe fn convert_function_arguments<'q>(
    ctx: *mut JSContext,
    js_this: JSValue,
    argc: c_int,
    argv: *mut JSValue,
) -> (Context<'q>, Value<'q>, Vec<Value<'q>>) {
    let (rctx, rthis, rargs) = qc::convert_function_arguments(ctx, js_this, argc, argv);
    let ctx = Context::from_raw(rctx);
    let this = Value::from_raw_parts(rthis, rctx);
    let args: Vec<Value> = rargs.iter().map(|v| Value::from_raw_parts(*v, rctx)).collect();
    (ctx, this, args)
}

/// This function is used by js_c_function macro.
#[inline]
pub fn convert_function_result(res: Value) -> JSValue {
    res.as_raw().as_js_value()
}
