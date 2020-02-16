#[macro_export]
macro_rules! js_c_function {
    ($f: expr) => {{
        unsafe extern "C" fn wrap(
            ctx: *mut $crate::core::ffi::JSContext,
            this_val: $crate::core::ffi::JSValue,
            argc: ::std::os::raw::c_int,
            argv: *mut $crate::core::ffi::JSValue,
        ) -> $crate::core::ffi::JSValue {
            use ::std::ptr::NonNull;
            let ctx = $crate::core::QjContext::new(NonNull::new(ctx).unwrap());
            let this_val = $crate::core::QjValue::new(this_val);
            let values = $crate::core::util::to_args(ctx, argc, argv);
            let ret = $f(ctx, this_val, values);
            $crate::core::conversion::AsJSValue::as_js_value(&ret)
        }
        Some(wrap)
    }};
}

#[macro_export]
macro_rules! js_class_finalizer {
    ($f: expr) => {{
        unsafe extern "C" fn wrap(rt: *mut $crate::core::ffi::JSRuntime, val: $crate::core::ffi::JSValue) {
            use ::std::ptr::NonNull;
            let rt = $crate::core::QjRuntime::new(NonNull::new(rt).unwrap());
            let val = $crate::core::QjValue::new(val);
            $f(rt, val)
        }
        Some(wrap)
    }};
}

#[macro_export]
macro_rules! js_class_gc_mark {
    ($f: expr) => {{
        unsafe extern "C" fn wrap(
            rt: *mut $crate::core::ffi::JSRuntime,
            val: $crate::core::ffi::JSValue,
            mark_func: $crate::core::ffi::JS_MarkFunc,
        ) {
            use ::std::ptr::NonNull;
            let rt = $crate::core::QjRuntime::new(NonNull::new(rt).unwrap());
            let val = $crate::core::QjValue::new(val);
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
            ctx: *mut $crate::core::ffi::JSContext,
            func_obj: $crate::core::ffi::JSValue,
            this_val: $crate::core::ffi::JSValue,
            argc: ::std::os::raw::c_int,
            argv: *mut $crate::core::ffi::JSValue,
            flags: ::std::os::raw::c_int,
        ) -> $crate::core::ffi::JSValue {
            use ::std::ptr::NonNull;
            let ctx = $crate::core::QjContext::new(NonNull::new(ctx).unwrap());
            let func_obj = $crate::core::QjValue::new(func_obj);
            let this_val = $crate::core::QjValue::new(this_val);
            let values = $crate::core::util::to_args(ctx, argc, argv);
            let ret = $f(ctx, func_obj, this_val, values, flags);
            $crate::core::conversion::AsJSValue::as_js_value(&ret)
        }
        Some(wrap)
    }};
}
