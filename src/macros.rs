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
            let (ctx, this, args) = $crate::convert_function_arguments::<'q>(ctx, this_val, argc, argv);
            let ret = $f(ctx, this, args.as_slice());
            $crate::convert_function_result(ret)
        }
        Some(wrap)
    }};
}

#[test]
fn test_js_c_function() {
    use crate::Context;
    let _f = js_c_function!(|ctx: Context<'q>, _this, _args| { ctx.new_int32(42).into() });
}
