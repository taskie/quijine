#[macro_export]
macro_rules! js_c_function {
    ($f: expr) => {{
        unsafe extern "C" fn wrap<'q>(
            ctx: *mut $crate::raw::JSContext,
            this_val: $crate::raw::JSValue,
            argc: ::std::os::raw::c_int,
            argv: *mut $crate::raw::JSValue,
        ) -> $crate::raw::JSValue {
            let f: fn($crate::Context<'q>, $crate::Value<'q>, &[$crate::Value<'q>]) -> $crate::Value<'q> = $f;
            let (ctx, this, args) = $crate::convert_function_arguments::<'q>(ctx, this_val, argc, argv);
            let ret = f(ctx, this, args.as_slice());
            $crate::convert_function_result(ret)
        }
        Some(wrap)
    }};
}

#[macro_export]
macro_rules! js_class_finalizer {
    ($f: expr) => {{
        unsafe extern "C" fn wrap<'r>(rt: *mut $crate::raw::JSRuntime, val: $crate::raw::JSValue) {
            let f: unsafe fn($crate::Runtime<'r>, $crate::Value<'r>) = $f;
            let rt = $crate::Runtime::from_raw(rt);
            let val = $crate::Value::from_raw_with_runtime(val, rt);
            f(rt, val)
        }
        Some(wrap)
    }};
}

#[macro_export]
macro_rules! js_class_gc_mark {
    ($f: expr) => {{
        unsafe extern "C" fn wrap<'r>(
            rt: *mut $crate::raw::JSRuntime,
            val: $crate::raw::JSValue,
            mark_func: $crate::raw::JS_MarkFunc,
        ) {
            let f: fn($crate::Runtime<'r>, $crate::Value<'r>, $crate::raw::JS_MarkFunc) = $f;
            let rt = $crate::Runtime::from_raw(rt);
            let val = $crate::Value::from_raw_with_runtime(val, rt);
            f(rt, val, mark_func)
        }
        Some(wrap)
    }};
}

#[macro_export]
macro_rules! js_class_call {
    ($f: expr) => {{
        unsafe extern "C" fn wrap<'q>(
            ctx: *mut $crate::raw::JSContext,
            func_obj: $crate::raw::JSValue,
            this_val: $crate::raw::JSValue,
            argc: ::std::os::raw::c_int,
            argv: *mut $crate::raw::JSValue,
            flags: ::std::os::raw::c_int,
        ) -> $crate::raw::JSValue {
            let f: fn(
                $crate::Context<'q>,
                $crate::Value<'q>,
                $crate::Value<'q>,
                &[$crate::Value<'q>],
                ::std::os::raw::c_int,
            ) -> $crate::Value<'q> = $f;
            let (ctx, this, args) = $crate::convert_function_arguments::<'q>(ctx, this_val, argc, argv);
            let func_obj = $crate::Value::from_raw(func_obj, ctx);
            let ret = f(ctx, func_obj, this, args.as_slice(), flags);
            $crate::convert_function_result(ret)
        }
        Some(wrap)
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! js_module_init_func {
    ($f: expr) => {{
        unsafe extern "C" fn wrap<'q>(
            ctx: *mut $crate::raw::JSContext,
            m: *mut $crate::raw::JSModuleDef,
        ) -> ::std::os::raw::c_int {
            let f: fn($crate::Context<'q>, $crate::ModuleDef<'q>) -> i32 = $f;
            let ctx = $crate::Context::from_raw(ctx);
            let m = $crate::ModuleDef::from_raw(m, ctx);
            f(ctx, m) as ::std::os::raw::c_int
        }
        Some(wrap)
    }};
}

#[cfg(test)]
mod tests {
    use crate::{raw, CFunctionListEntry, ClassDef, Value};
    use std::{ffi::CString, ptr::null_mut};

    #[test]
    fn test() {
        let class_name = CString::new("Test").unwrap();
        let f_name = CString::new("Test").unwrap();
        let _f = unsafe {
            CFunctionListEntry::cfunc_def(&f_name, 0, js_c_function!(|_ctx, _this, _args| { Value::undefined() }))
        };
        let _class_def = unsafe {
            ClassDef::from_raw(raw::JSClassDef {
                class_name: class_name.as_ptr(),
                finalizer: js_class_finalizer!(|_rt, _val| {}),
                gc_mark: js_class_gc_mark!(|_rt, _val, _f| {}),
                call: js_class_call!(|_ctx, _f, _this, _args, _flags| { Value::undefined() }),
                exotic: null_mut(),
            })
        };
        let _f = js_module_init_func!(|_ctx, _m| 0);
    }
}
