use crate::{
    core::{conversion::AsJSValue, ffi, Context, Value},
    error::{QjError, QjErrorValue, QjResult},
    instance::QjVec,
    runtime::QjRuntime,
    tags::{QjAnyTag, QjIntTag, QjNullTag, QjObjectTag, QjStringTag, QjUndefinedTag},
    Qj, QjEvalFlags, QjRuntimeGuard,
};
use std::{fmt, os::raw::c_int};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct QjContext<'q>(Context<'q>);

impl<'q> QjContext<'q> {
    #[inline]
    pub(crate) fn from(ctx: Context<'q>) -> Self {
        QjContext(ctx)
    }

    #[inline]
    pub(crate) fn into(self) -> Context<'q> {
        self.0
    }

    #[inline]
    fn wrap_result<T>(self, val: Value<'q>) -> QjResult<'q, Qj<'q, T>> {
        if val.is_exception() {
            Err(QjError::from_value(Qj::<QjAnyTag>::from(self.0.exception(), self.0)))
        } else {
            Ok(Qj::<QjAnyTag>::from(val, self.0))
        }
    }

    #[inline]
    pub fn eval(self, code: &str, filename: &str, eval_flags: QjEvalFlags) -> QjResult<'q, Qj<'q, QjAnyTag>> {
        self.wrap_result(self.0.eval(code, filename, eval_flags))
    }

    #[inline]
    pub fn call<F, T>(self, func_obj: F, this_obj: T, args: &QjVec<'q, QjAnyTag>) -> QjResult<'q, Qj<'q, QjAnyTag>>
    where
        F: AsRef<Qj<'q, QjAnyTag>>,
        T: AsRef<Qj<'q, QjAnyTag>>,
    {
        let val = self.0.call(
            func_obj.as_ref().as_value(),
            this_obj.as_ref().as_value(),
            args.as_slice(),
        );
        self.wrap_result(val)
    }

    #[inline]
    pub fn global_object(self) -> Qj<'q, QjObjectTag> {
        self.wrap_result(self.0.global_object()).unwrap()
    }

    #[inline]
    pub fn new_object(self) -> Qj<'q, QjObjectTag> {
        self.wrap_result(self.0.new_object()).unwrap()
    }

    #[inline]
    pub fn new_int32(self, v: i32) -> Qj<'q, QjIntTag> {
        self.wrap_result(self.0.new_int32(v)).unwrap()
    }

    #[inline]
    pub fn new_int64(self, v: i64) -> Qj<'q, QjIntTag> {
        self.wrap_result(self.0.new_int64(v)).unwrap()
    }

    #[inline]
    pub fn new_float64(self, v: f64) -> Qj<'q, QjIntTag> {
        self.wrap_result(self.0.new_float64(v)).unwrap()
    }

    #[inline]
    pub fn new_string(self, v: &str) -> Qj<'q, QjStringTag> {
        self.wrap_result(self.0.new_string(v)).unwrap()
    }

    #[inline]
    pub fn new_string_from_bytes(self, v: &[u8]) -> Qj<'q, QjStringTag> {
        self.wrap_result(self.0.new_string_from_bytes(v)).unwrap()
    }

    // callback

    #[inline]
    pub fn new_function<F>(self, func: F, name: &str, length: i32) -> Qj<'q, QjObjectTag>
    where
        F: 'static + Send + Fn(QjContext<'q>, Qj<'q, QjAnyTag>, QjVec<'q, QjAnyTag>) -> QjResult<'q, Qj<'q, QjAnyTag>>,
    {
        self.new_callback(Box::new(move |ctx, this, args| func(ctx, this, args)), name, length)
    }

    #[inline]
    pub fn new_callback(self, func: QjCallback<'q, 'static>, _name: &str, length: i32) -> Qj<'q, QjObjectTag> {
        unsafe extern "C" fn call(
            ctx: *mut ffi::JSContext,
            js_this: ffi::JSValue,
            argc: c_int,
            argv: *mut ffi::JSValue,
            _magic: c_int,
            func_data: *mut ffi::JSValue,
        ) -> ffi::JSValue {
            let ctx = Context::from_ptr(ctx);
            let this = Value::from_raw(js_this, ctx);
            let mut args: Vec<Value> = Vec::with_capacity(argc as usize);
            for i in 0..argc {
                let p = argv.offset(i as isize);
                let any = Value::from_raw(*p, ctx);
                args.push(any);
            }
            let cb = Value::from_raw(*func_data, ctx);
            log::debug!("load pointer from ArrayBuffer");
            let func = ctx.array_buffer_to_sized::<QjCallback>(&cb).unwrap();

            log::debug!("this");
            let this = Qj::<QjAnyTag>::from(this, ctx);
            Qj::dup(&this);
            log::debug!("args");
            let args = QjVec::<QjAnyTag>::from(args.as_slice(), ctx);
            QjVec::dup(&args);
            let ctx = QjContext::from(ctx);

            log::debug!("invoke start");
            let r = (*func)(ctx, this, args);
            let res = match r {
                Ok(t) => {
                    Qj::dup(&t);
                    t.as_value().as_js_value()
                }
                Err(e) => {
                    let v = e.value;
                    match v {
                        QjErrorValue::None => ctx.0.throw(ctx.0.new_string("some error occured")),
                        QjErrorValue::String(s) => ctx.0.throw(ctx.0.new_string(s)),
                        QjErrorValue::Value(v) => {
                            Qj::dup(&v);
                            ctx.0.throw(v.as_value())
                        }
                    };
                    Value::exception().as_js_value()
                }
            };
            log::debug!("invoke end");
            res
        };
        unsafe {
            log::debug!("save pointer to ArrayBuffer");
            let cb = self.0.new_array_buffer_copy_from_sized::<QjCallback>(func);
            log::debug!("new c function data");
            let cfd = self.0.new_c_function_data(Some(call), length, 0, vec![cb]);
            self.0.free_value(cb);
            Qj::<QjAnyTag>::from(cfd, self.0)
        }
    }

    // special values

    pub fn undefined(self) -> Qj<'q, QjUndefinedTag> {
        Qj::<QjUndefinedTag>::from(Value::undefined(), self.0)
    }

    pub fn null(self) -> Qj<'q, QjNullTag> {
        Qj::<QjNullTag>::from(Value::null(), self.0)
    }
}

pub struct QjContextGuard<'r>(QjContext<'r>);

impl<'r> QjContextGuard<'r> {
    pub fn new(rt: QjRuntime) -> QjContextGuard {
        QjContextGuard(QjContext(Context::new(rt.into())))
    }

    pub fn new_with_guard(rtg: &QjRuntimeGuard) -> QjContextGuard {
        QjContextGuard(QjContext(Context::new(rtg.get().into())))
    }

    #[inline]
    pub fn get(&self) -> QjContext<'r> {
        self.0
    }

    #[inline]
    pub fn with<F, R>(&'r self, f: F) -> R
    where
        F: FnOnce(QjContext<'r>) -> R,
    {
        f(self.0)
    }
}

impl Drop for QjContextGuard<'_> {
    fn drop(&mut self) {
        unsafe { Context::free(self.0.into()) }
    }
}

impl fmt::Debug for QjContextGuard<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.write_str(format!("ContextGuard({:?})", self.0).as_str())
    }
}

pub(crate) type QjCallback<'q, 'a> =
    Box<dyn Fn(QjContext<'q>, Qj<'q, QjAnyTag>, QjVec<'q, QjAnyTag>) -> QjResult<'q, Qj<'q, QjAnyTag>> + 'a>;
