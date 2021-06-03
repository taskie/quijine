#[macro_export]
macro_rules! js_c_function {
    ($f: expr) => {{
        unsafe extern "C" fn wrap(
            ctx: *mut $crate::raw::JSContext,
            this_val: $crate::raw::JSValue,
            argc: ::std::os::raw::c_int,
            argv: *mut $crate::raw::JSValue,
        ) -> $crate::raw::JSValue {
            let (ctx, this, args) = $crate::convert_function_arguments(ctx, this_val, argc, argv);
            let ret = $f(ctx, this, args.as_slice());
            $crate::convert_function_result(&ret)
        }
        Some(wrap)
    }};
}

#[macro_export]
macro_rules! js_class_finalizer {
    ($f: expr) => {{
        unsafe extern "C" fn wrap(rt: *mut $crate::raw::JSRuntime, val: $crate::raw::JSValue) {
            let rt = $crate::Runtime::from_raw(rt);
            let val = $crate::Value::from_raw_with_runtime(val, rt);
            $f(rt, val)
        }
        Some(wrap)
    }};
}

#[macro_export]
macro_rules! js_class_gc_mark {
    ($f: expr) => {{
        unsafe extern "C" fn wrap(
            rt: *mut $crate::raw::JSRuntime,
            val: $crate::raw::JSValue,
            mark_func: $crate::raw::JS_MarkFunc,
        ) {
            let rt = $crate::Runtime::from_raw(rt);
            let val = $crate::Value::from_raw_with_runtime(val, rt);
            $f(rt, val, mark_func)
        }
        Some(wrap)
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! js_class_call {
    ($f: expr) => {{
        unsafe extern "C" fn wrap(
            ctx: *mut $crate::raw::JSContext,
            func_obj: $crate::raw::JSValue,
            this_val: $crate::raw::JSValue,
            argc: ::std::os::raw::c_int,
            argv: *mut $crate::raw::JSValue,
            flags: ::std::os::raw::c_int,
        ) -> $crate::raw::JSValue {
            let (ctx, this, args) = $crate::convert_function_arguments(ctx, this_val, argc, argv);
            let func_obj = $crate::Value::from_raw(func_obj, ctx);
            let ret = $f(ctx, func_obj, this, args.as_slice(), flags);
            $crate::convert_function_result(&ret)
        }
        Some(wrap)
    }};
}
