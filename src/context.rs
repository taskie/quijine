use crate::{
    class::QjClass,
    class_util::register_class,
    error::{QjError, QjErrorValue, QjResult},
    runtime::QjRuntime,
    tags::{QjAnyTag, QjBoolTag, QjFloat64Tag, QjIntTag, QjNullTag, QjObjectTag, QjStringTag, QjUndefinedTag},
    Qj, QjEvalFlags, QjRuntimeGuard,
};
use qjncore::{conversion::AsJsValue, raw, ClassId, Context, Value};
use std::{any::TypeId, collections::HashSet, ffi::c_void, fmt, os::raw::c_int};

pub struct QjContextOpaque {
    registered_classes: HashSet<TypeId>,
}

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
    pub fn runtime(self) -> QjRuntime<'q> {
        QjRuntime::from(self.0.runtime())
    }

    #[inline]
    pub(crate) fn opaque(&self) -> &QjContextOpaque {
        unsafe { &*(self.0.opaque() as *mut QjContextOpaque) }
    }

    #[inline]
    pub(crate) fn opaque_mut(&mut self) -> &mut QjContextOpaque {
        unsafe { &mut *(self.0.opaque() as *mut QjContextOpaque) }
    }

    #[inline]
    fn wrap_result<T>(self, val: Value<'q>) -> QjResult<'q, Qj<'q, T>> {
        if val.is_exception() {
            Err(QjError::with_value(Qj::<QjAnyTag>::from(self.0.exception(), self.0)))
        } else {
            Ok(Qj::from(val, self.0))
        }
    }

    #[inline]
    pub fn eval(self, code: &str, filename: &str, eval_flags: QjEvalFlags) -> QjResult<'q, Qj<'q, QjAnyTag>> {
        self.wrap_result(self.0.eval(code, filename, eval_flags))
    }

    #[inline]
    pub fn call<F, T, A>(self, func_obj: F, this_obj: T, args: A) -> QjResult<'q, Qj<'q, QjAnyTag>>
    where
        F: AsRef<Qj<'q, QjAnyTag>>,
        T: AsRef<Qj<'q, QjAnyTag>>,
        A: AsRef<[Qj<'q, QjAnyTag>]>,
    {
        let qc_args: Vec<_> = args.as_ref().iter().map(|v| v.as_value()).collect();
        let val = self
            .0
            .call(func_obj.as_ref().as_value(), this_obj.as_ref().as_value(), qc_args);
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
    pub fn new_object_class<C: QjClass + 'static>(mut self) -> Qj<'q, QjObjectTag> {
        let clz = self.register_class::<C>();
        self.wrap_result(self.0.new_object_class(clz)).unwrap()
    }

    #[inline]
    pub fn new_bool(self, v: bool) -> Qj<'q, QjBoolTag> {
        self.wrap_result(self.0.new_bool(v)).unwrap()
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
    pub fn new_float64(self, v: f64) -> Qj<'q, QjFloat64Tag> {
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
        F: 'static + Send + Fn(QjContext<'q>, Qj<'q, QjAnyTag>, &[Qj<'q, QjAnyTag>]) -> QjResult<'q, Qj<'q, QjAnyTag>>,
    {
        self.new_callback(Box::new(move |ctx, this, args| func(ctx, this, args)), name, length)
    }

    #[inline]
    pub fn new_callback(self, func: QjCallback<'q, 'static>, _name: &str, length: i32) -> Qj<'q, QjObjectTag> {
        unsafe extern "C" fn call(
            ctx: *mut raw::JSContext,
            js_this: raw::JSValue,
            argc: c_int,
            argv: *mut raw::JSValue,
            _magic: c_int,
            func_data: *mut raw::JSValue,
        ) -> raw::JSValue {
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
            let func = cb.array_buffer_to_sized::<QjCallback>(ctx).unwrap();

            log::debug!("this");
            let this = Qj::<QjAnyTag>::from(this, ctx);
            Qj::dup(&this);
            log::debug!("args");
            let args: Vec<_> = args.iter().map(|v| Qj::<QjAnyTag>::from(*v, ctx)).collect();
            args.iter().for_each(Qj::dup);
            let ctx = QjContext::from(ctx);

            log::debug!("invoke start");
            let r = (*func)(ctx, this, args.as_slice());
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
        }
        unsafe {
            log::debug!("save pointer to ArrayBuffer");
            let cb = self.0.new_array_buffer_copy_from_sized::<QjCallback>(func);
            log::debug!("new c function data");
            let cfd = self.0.new_c_function_data(Some(call), length, 0, vec![cb]);
            self.0.free_value(cb);
            Qj::from(cfd, self.0)
        }
    }

    // special values

    pub fn undefined(self) -> Qj<'q, QjUndefinedTag> {
        Qj::<QjUndefinedTag>::from(Value::undefined(), self.0)
    }

    pub fn null(self) -> Qj<'q, QjNullTag> {
        Qj::<QjNullTag>::from(Value::null(), self.0)
    }

    // json

    pub fn parse_json(self, buf: &str, filename: &str) -> QjResult<'q, Qj<'q, QjAnyTag>> {
        self.wrap_result(self.0.parse_json(buf, filename))
    }

    pub fn json_stringify(
        self,
        obj: Qj<'q, QjAnyTag>,
        replacer: Qj<'q, QjAnyTag>,
        space0: Qj<'q, QjAnyTag>,
    ) -> QjResult<'q, Qj<'q, QjStringTag>> {
        self.wrap_result(
            self.0
                .json_stringify(obj.as_value(), replacer.as_value(), space0.as_value()),
        )
    }

    // class

    pub(crate) fn register_class<T: 'static + QjClass>(&mut self) -> ClassId {
        let type_id = TypeId::of::<T>();
        let class_id = self.runtime().get_or_register_class_id::<T>();
        if self.opaque().registered_classes.contains(&type_id) {
            return class_id;
        }
        register_class::<T>(self.0, class_id);
        self.opaque_mut().registered_classes.insert(type_id);
        class_id
    }
}

pub struct QjContextGuard<'r>(QjContext<'r>);

impl<'r> QjContextGuard<'r> {
    pub fn new(rt: QjRuntime) -> QjContextGuard {
        let ctx = Context::new(rt.into());
        let opaque = Box::new(QjContextOpaque {
            registered_classes: HashSet::new(),
        });
        unsafe {
            ctx.set_opaque(Box::into_raw(opaque) as *mut c_void);
        }
        QjContextGuard(QjContext(ctx))
    }

    pub fn new_with_guard(rtg: &QjRuntimeGuard) -> QjContextGuard {
        QjContextGuard::new(rtg.get())
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
        unsafe {
            Box::from_raw((self.0).0.opaque() as *mut QjContextOpaque);
            Context::free(self.0.into())
        }
    }
}

impl fmt::Debug for QjContextGuard<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.write_str(format!("ContextGuard({:?})", self.0).as_str())
    }
}

pub(crate) type QjCallback<'q, 'a> =
    Box<dyn Fn(QjContext<'q>, Qj<'q, QjAnyTag>, &[Qj<'q, QjAnyTag>]) -> QjResult<'q, Qj<'q, QjAnyTag>> + 'a>;
