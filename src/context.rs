use crate::{
    class::QjClass,
    class_util::register_class,
    error::{QjError, QjErrorValue, QjResult},
    runtime::QjRuntime,
    types::{Bool, Float64, Int, Null, Object, String as QjString, Undefined},
    Data, QjEvalFlags, QjRuntimeGuard,
};
use qjncore::{conversion::AsJsValue, raw, ClassId, Context, Value};
use std::{any::TypeId, collections::HashSet, convert::TryInto, ffi::c_void, fmt, os::raw::c_int};

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
    fn wrap_result(self, val: Value<'q>) -> QjResult<'q, Data<'q>> {
        if val.is_exception() {
            Err(QjError::with_value(Data::from(self.0.exception(), self.0)))
        } else {
            Ok(Data::from(val, self.0))
        }
    }

    #[inline]
    pub fn eval(self, code: &str, filename: &str, eval_flags: QjEvalFlags) -> QjResult<'q, Data<'q>> {
        self.wrap_result(self.0.eval(code, filename, eval_flags))
    }

    #[inline]
    pub fn call<F, T, A>(self, func_obj: F, this_obj: T, args: A) -> QjResult<'q, Data<'q>>
    where
        F: AsRef<Data<'q>>,
        T: AsRef<Data<'q>>,
        A: AsRef<[Data<'q>]>,
    {
        let qc_args: Vec<_> = args.as_ref().iter().map(|v| v.as_value()).collect();
        let val = self
            .0
            .call(func_obj.as_ref().as_value(), this_obj.as_ref().as_value(), qc_args);
        self.wrap_result(val)
    }

    #[inline]
    pub fn global_object(self) -> Object<'q> {
        self.wrap_result(self.0.global_object()).unwrap().try_into().unwrap()
    }

    #[inline]
    pub fn new_object(self) -> Object<'q> {
        self.wrap_result(self.0.new_object()).unwrap().try_into().unwrap()
    }

    #[inline]
    pub fn new_object_class<C: QjClass + 'static>(mut self) -> Object<'q> {
        let clz = self.register_class::<C>();
        self.wrap_result(self.0.new_object_class(clz))
            .unwrap()
            .try_into()
            .unwrap()
    }

    #[inline]
    pub fn new_bool(self, v: bool) -> Bool<'q> {
        self.wrap_result(self.0.new_bool(v)).unwrap().try_into().unwrap()
    }

    #[inline]
    pub fn new_int32(self, v: i32) -> Int<'q> {
        self.wrap_result(self.0.new_int32(v)).unwrap().try_into().unwrap()
    }

    #[inline]
    pub fn new_int64(self, v: i64) -> Data<'q> {
        self.wrap_result(self.0.new_int64(v)).unwrap()
    }

    #[inline]
    pub fn new_float64(self, v: f64) -> Float64<'q> {
        self.wrap_result(self.0.new_float64(v)).unwrap().try_into().unwrap()
    }

    #[inline]
    pub fn new_string(self, v: &str) -> QjString<'q> {
        self.wrap_result(self.0.new_string(v)).unwrap().try_into().unwrap()
    }

    // callback

    #[inline]
    pub fn new_function<F>(self, func: F, name: &str, length: i32) -> Object<'q>
    where
        F: 'static + Send + Fn(QjContext<'q>, Data<'q>, &[Data<'q>]) -> QjResult<'q, Data<'q>>,
    {
        self.new_callback(Box::new(move |ctx, this, args| func(ctx, this, args)), name, length)
    }

    #[inline]
    pub fn new_callback(self, func: QjCallback<'q, 'static>, _name: &str, length: i32) -> Object<'q> {
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
            let this = Data::from(this, ctx);
            Data::dup(&this);
            log::debug!("args");
            let args: Vec<_> = args.iter().map(|v| Data::from(*v, ctx)).collect();
            args.iter().for_each(Data::dup);
            let ctx = QjContext::from(ctx);

            log::debug!("invoke start");
            let r = (*func)(ctx, this, args.as_slice());
            let res = match r {
                Ok(t) => {
                    Data::dup(&t);
                    t.as_value().as_js_value()
                }
                Err(e) => {
                    let v = e.value;
                    match v {
                        QjErrorValue::None => ctx.0.throw(ctx.0.new_string("some error occured")),
                        QjErrorValue::String(s) => ctx.0.throw(ctx.0.new_string(s)),
                        QjErrorValue::Value(v) => {
                            Data::dup(&v);
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
            Data::from(cfd, self.0).try_into().unwrap()
        }
    }

    // special values

    pub fn undefined(self) -> Undefined<'q> {
        Data::from(Value::undefined(), self.0).try_into().unwrap()
    }

    pub fn null(self) -> Null<'q> {
        Data::from(Value::null(), self.0).try_into().unwrap()
    }

    // json

    pub fn parse_json(self, buf: &str, filename: &str) -> QjResult<'q, Data<'q>> {
        self.wrap_result(self.0.parse_json(buf, filename))
    }

    pub fn json_stringify(self, obj: Data<'q>, replacer: Data<'q>, space0: Data<'q>) -> QjResult<'q, Data<'q>> {
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

pub(crate) type QjCallback<'q, 'a> = Box<dyn Fn(QjContext<'q>, Data<'q>, &[Data<'q>]) -> QjResult<'q, Data<'q>> + 'a>;
