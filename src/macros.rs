#[macro_export]
macro_rules! qj_slice {
    [ $($v:expr),* ] => {
        &[$(Into::<$crate::Value>::into($v)),*]
    };
}

#[macro_export]
macro_rules! qj_vec {
    [ $($v:expr),* ] => {
        vec![$(Into::<$crate::Value>::into($v)),*]
    };
}

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
            let (ctx, this, args) = $crate::internal::convert_function_arguments::<'q>(ctx, this_val, argc, argv);
            let ret = f(ctx, this, args.as_slice());
            $crate::internal::convert_function_result(ret)
        }
        Some(wrap)
    }};
}

#[macro_export]
macro_rules! js_c_getter {
    ($f: expr) => {{
        unsafe extern "C" fn wrap<'q>(
            ctx: *mut $crate::raw::JSContext,
            this: $crate::raw::JSValue,
        ) -> $crate::raw::JSValue {
            let f: fn($crate::Context<'q>, $crate::Value<'q>) -> $crate::Value<'q> = $f;
            let ctx = $crate::internal::convert_context(ctx);
            let this = $crate::internal::convert_value_and_dup(this, ctx);
            let ret = f(ctx, this);
            $crate::internal::convert_function_result(ret)
        }
        Some(wrap)
    }};
}

#[macro_export]
macro_rules! js_c_setter {
    ($f: expr) => {{
        unsafe extern "C" fn wrap<'q>(
            ctx: *mut $crate::raw::JSContext,
            this: $crate::raw::JSValue,
            val: $crate::raw::JSValue,
        ) -> $crate::raw::JSValue {
            let f: fn($crate::Context<'q>, $crate::Value<'q>, $crate::Value<'q>) -> $crate::Value<'q> = $f;
            let ctx = $crate::internal::convert_context(ctx);
            let this = $crate::internal::convert_value_and_dup(this, ctx);
            let val = $crate::internal::convert_value_and_dup(val, ctx);
            let ret = f(ctx, this, val);
            $crate::internal::convert_function_result(ret)
        }
        Some(wrap)
    }};
}

#[macro_export]
macro_rules! js_module_init_func {
    ($f: expr) => {{
        unsafe extern "C" fn wrap<'q>(
            ctx: *mut $crate::raw::JSContext,
            m: *mut $crate::raw::JSModuleDef,
        ) -> ::std::os::raw::c_int {
            let f: fn($crate::Context<'q>, $crate::ModuleDef<'q>) -> i32 = $f;
            let ctx = $crate::internal::convert_context(ctx);
            let m = $crate::internal::convert_module_def(m, ctx);
            f(ctx, m) as ::std::os::raw::c_int
        }
        Some(wrap)
    }};
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {
        let _f = js_c_function!(|ctx, _this, _args| { ctx.undefined().into() });
        let _f = js_c_getter!(|ctx, _this| { ctx.undefined().into() });
        let _f = js_c_setter!(|ctx, _this, _val| { ctx.undefined().into() });
        let _f = js_module_init_func!(|_ctx, _m| 0);
    }
}
