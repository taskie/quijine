use crate::{conversion::AsJsValue, ffi, Context, Value};
use std::{os::raw::c_int, slice};

#[inline]
pub unsafe fn convert_function_arguments<'q>(
    ctx: *mut ffi::JSContext,
    js_this: ffi::JSValue,
    argc: c_int,
    argv: *mut ffi::JSValue,
) -> (Context<'q>, Value<'q>, Vec<Value<'q>>) {
    let ctx = Context::from_ptr(ctx);
    let this = Value::from_raw(js_this, ctx);
    let args = slice::from_raw_parts(argv, argc as usize);
    let args: Vec<Value> = args.into_iter().map(|v| Value::from_raw(*v, ctx)).collect();
    (ctx, this, args)
}

#[inline]
pub fn convert_function_result<'q>(res: &Value) -> ffi::JSValue {
    res.as_js_value()
}
