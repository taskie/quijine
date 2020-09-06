#[macro_export]
macro_rules! js_c_function {
    ($f: expr) => {{
        unsafe extern "C" fn wrap(
            ctx: *mut crate::ffi::JSContext,
            this_val: crate::ffi::JSValue,
            argc: ::std::os::raw::c_int,
            argv: *mut crate::ffi::JSValue,
        ) -> crate::ffi::JSValue {
            let ctx = crate::Context::from_ptr(ctx);
            let this_val = crate::Value::from_raw(this_val, ctx);
            let values = ::std::slice::from_raw_parts(argv, argc as usize);
            let values: Vec<Value> = values.iter().map(|v| Value::from_raw(*v, ctx)).collect();
            let ret = $f(ctx, this_val, values.as_slice());
            crate::conversion::AsJSValue::as_js_value(&ret)
        }
        Some(wrap)
    }};
}

#[macro_export]
macro_rules! js_class_finalizer {
    ($f: expr) => {{
        unsafe extern "C" fn wrap(rt: *mut crate::ffi::JSRuntime, val: $crate::ffi::JSValue) {
            let rt = crate::Runtime::from_ptr(rt);
            let val = crate::Value::from_raw_with_runtime(val, rt);
            $f(rt, val)
        }
        Some(wrap)
    }};
}

#[macro_export]
macro_rules! js_class_gc_mark {
    ($f: expr) => {{
        unsafe extern "C" fn wrap(
            rt: *mut crate::ffi::JSRuntime,
            val: crate::ffi::JSValue,
            mark_func: crate::ffi::JS_MarkFunc,
        ) {
            let rt = crate::Runtime::from_ptr(rt);
            let val = crate::Value::from_raw_with_runtime(val, rt);
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
            ctx: *mut crate::ffi::JSContext,
            func_obj: crate::ffi::JSValue,
            this_val: crate::ffi::JSValue,
            argc: ::std::os::raw::c_int,
            argv: *mut crate::ffi::JSValue,
            flags: ::std::os::raw::c_int,
        ) -> crate::ffi::JSValue {
            let ctx = crate::Context::from_ptr(ctx);
            let func_obj = crate::Value::from_raw(func_obj, ctx);
            let this_val = crate::Value::from_raw(this_val, ctx);
            let values = ::std::slice::from_raw_parts(argv, argc as usize);
            let values: Vec<Value> = values.iter().map(|v| Value::from_raw(*v, ctx)).collect();
            let ret = $f(ctx, func_obj, this_val, values.as_slice(), flags);
            crate::conversion::AsJSValue::as_js_value(&ret)
        }
        Some(wrap)
    }};
}
