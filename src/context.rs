use crate::{
    class::Class,
    class_util::register_class,
    error::{Error, ErrorKind, ErrorValue, Result},
    runtime::Runtime,
    types::{Bool, Float64, Int, Null, Object, String as QjString, Undefined},
    Data, EvalFlags, RuntimeScope,
};
use qc::conversion::AsJsValue;
use qjncore::{self as qc, raw};
use std::{any::TypeId, collections::HashSet, convert::TryInto, ffi::c_void, fmt, os::raw::c_int};

pub struct ContextOpaque {
    registered_classes: HashSet<TypeId>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Context<'q>(qc::Context<'q>);

impl<'q> Context<'q> {
    #[inline]
    pub(crate) fn from(ctx: qc::Context<'q>) -> Self {
        Context(ctx)
    }

    #[inline]
    pub(crate) fn into(self) -> qc::Context<'q> {
        self.0
    }

    #[inline]
    pub fn runtime(self) -> Runtime<'q> {
        Runtime::from(self.0.runtime())
    }

    #[inline]
    pub(crate) fn opaque(&self) -> &ContextOpaque {
        unsafe { &*(self.0.opaque() as *mut ContextOpaque) }
    }

    #[inline]
    pub(crate) fn opaque_mut(&mut self) -> &mut ContextOpaque {
        unsafe { &mut *(self.0.opaque() as *mut ContextOpaque) }
    }

    #[inline]
    fn wrap_result(self, val: qc::Value<'q>) -> Result<Data<'q>> {
        if val.is_exception() {
            Err(Error::from_js_error(
                ErrorKind::InternalError,
                Data::from(self.0.exception(), self.0),
            ))
        } else {
            Ok(Data::from(val, self.0))
        }
    }

    #[inline]
    pub fn eval(self, code: &str, filename: &str, eval_flags: EvalFlags) -> Result<Data<'q>> {
        self.wrap_result(self.0.eval(code, filename, eval_flags))
    }

    #[inline]
    pub fn call<F, T, A>(self, func_obj: F, this_obj: T, args: A) -> Result<Data<'q>>
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
        unsafe { self.wrap_result(self.0.new_object()).unwrap().into_unchecked() }
    }

    #[inline]
    pub fn new_object_class<C: Class + 'static>(mut self) -> Object<'q> {
        let clz = self.register_class::<C>();
        unsafe { self.wrap_result(self.0.new_object_class(clz)).unwrap().into_unchecked() }
    }

    #[inline]
    pub fn new_object_with_opaque<C: Class + 'static>(self, v: Box<C>) -> Object<'q> {
        let mut obj = self.new_object_class::<C>();
        obj.set_opaque(v);
        obj
    }

    #[inline]
    pub fn new_bool(self, v: bool) -> Bool<'q> {
        unsafe { self.wrap_result(self.0.new_bool(v)).unwrap().into_unchecked() }
    }

    #[inline]
    pub fn new_int32(self, v: i32) -> Int<'q> {
        unsafe { self.wrap_result(self.0.new_int32(v)).unwrap().into_unchecked() }
    }

    #[inline]
    pub fn new_int64(self, v: i64) -> Data<'q> {
        unsafe { self.wrap_result(self.0.new_int64(v)).unwrap().into_unchecked() }
    }

    #[inline]
    pub fn new_float64(self, v: f64) -> Float64<'q> {
        unsafe { self.wrap_result(self.0.new_float64(v)).unwrap().into_unchecked() }
    }

    #[inline]
    pub fn new_string(self, v: &str) -> QjString<'q> {
        unsafe { self.wrap_result(self.0.new_string(v)).unwrap().into_unchecked() }
    }

    // callback

    #[inline]
    pub fn new_function<F, R>(self, func: F, name: &str, length: i32) -> Object<'q>
    where
        F: Fn(Context<'q>, Data<'q>, &[Data<'q>]) -> Result<R> + Send + 'static,
        R: Into<Data<'q>> + 'q,
    {
        self.new_callback(Box::new(move |ctx, this, args| func(ctx, this, args)), name, length)
    }

    #[inline]
    pub fn new_callback<R>(self, func: QjCallback<'q, 'static, R>, _name: &str, length: i32) -> Object<'q>
    where
        R: Into<Data<'q>> + 'q,
    {
        unsafe extern "C" fn call<'q, R: Into<Data<'q>>>(
            ctx: *mut raw::JSContext,
            js_this: raw::JSValue,
            argc: c_int,
            argv: *mut raw::JSValue,
            _magic: c_int,
            func_data: *mut raw::JSValue,
        ) -> raw::JSValue {
            let ctx = qc::Context::from_ptr(ctx);
            let this = qc::Value::from_raw(js_this, ctx);
            let mut args: Vec<qc::Value> = Vec::with_capacity(argc as usize);
            for i in 0..argc {
                let p = argv.offset(i as isize);
                let any = qc::Value::from_raw(*p, ctx);
                args.push(any);
            }
            let cb = qc::Value::from_raw(*func_data, ctx);
            log::debug!("load pointer from ArrayBuffer");
            let func = cb.array_buffer_to_sized::<QjCallback<R>>(ctx).unwrap();

            log::debug!("this");
            let this = Data::from(this, ctx);
            Data::dup(&this);
            log::debug!("args");
            let args: Vec<_> = args.iter().map(|v| Data::from(*v, ctx)).collect();
            args.iter().for_each(Data::dup);
            let ctx = Context::from(ctx);

            log::debug!("invoke start");
            let r = (*func)(ctx, this, args.as_slice());
            let res = match r {
                Ok(t) => {
                    let t = t.into();
                    Data::dup(&t);
                    t.as_value().as_js_value()
                }
                Err(e) => {
                    let v = e.value;
                    match v {
                        ErrorValue::None => ctx.0.throw(ctx.0.new_string("some error occured")),
                        ErrorValue::String(s) => ctx.0.throw(ctx.0.new_string(s)),
                        ErrorValue::JsError(e) => ctx.0.throw(ctx.0.new_string(format!("{}", e))),
                        ErrorValue::External(e) => ctx.0.throw(ctx.0.new_string(format!("{}", e))),
                    };
                    qc::Value::exception().as_js_value()
                }
            };
            log::debug!("invoke end");
            res
        }
        unsafe {
            log::debug!("save pointer to ArrayBuffer");
            let cb = self.0.new_array_buffer_copy_from_sized(func);
            log::debug!("new c function data");
            let cfd = self.0.new_c_function_data(Some(call::<R>), length, 0, vec![cb]);
            self.0.free_value(cb);
            Data::from(cfd, self.0).into_unchecked()
        }
    }

    // special values

    pub fn undefined(self) -> Undefined<'q> {
        unsafe { Data::from(qc::Value::undefined(), self.0).into_unchecked() }
    }

    pub fn null(self) -> Null<'q> {
        unsafe { Data::from(qc::Value::null(), self.0).into_unchecked() }
    }

    // json

    pub fn parse_json(self, buf: &str, filename: &str) -> Result<Data<'q>> {
        self.wrap_result(self.0.parse_json(buf, filename))
    }

    pub fn json_stringify(
        self,
        obj: impl AsRef<Data<'q>>,
        replacer: impl AsRef<Data<'q>>,
        space0: impl AsRef<Data<'q>>,
    ) -> Result<QjString<'q>> {
        self.wrap_result(self.0.json_stringify(
            obj.as_ref().as_value(),
            replacer.as_ref().as_value(),
            space0.as_ref().as_value(),
        ))
        .map(|v| unsafe { v.into_unchecked() })
    }

    // class

    pub(crate) fn register_class<T: 'static + Class>(&mut self) -> qc::ClassId {
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

pub struct ContextScope<'r>(Context<'r>);

impl<'r> ContextScope<'r> {
    pub fn new(rt: Runtime) -> ContextScope {
        let ctx = qc::Context::new(rt.into());
        let opaque = Box::new(ContextOpaque {
            registered_classes: HashSet::new(),
        });
        unsafe {
            ctx.set_opaque(Box::into_raw(opaque) as *mut c_void);
        }
        ContextScope(Context(ctx))
    }

    pub fn new_with_scope(rts: &RuntimeScope) -> ContextScope {
        ContextScope::new(rts.get())
    }

    #[inline]
    pub fn get(&self) -> Context<'r> {
        self.0
    }

    #[inline]
    pub fn with<F, R>(&'r self, f: F) -> Result<R>
    where
        F: FnOnce(Context<'r>) -> Result<R>,
    {
        f(self.0)
    }
}

impl Drop for ContextScope<'_> {
    fn drop(&mut self) {
        unsafe {
            Box::from_raw((self.0).0.opaque() as *mut ContextOpaque);
            qc::Context::free(self.0.into())
        }
    }
}

impl fmt::Debug for ContextScope<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::result::Result<(), fmt::Error> {
        f.write_str(format!("ContextScope({:?})", self.0).as_str())
    }
}

pub(crate) type QjCallback<'q, 'a, R> = Box<dyn Fn(Context<'q>, Data<'q>, &[Data<'q>]) -> Result<R> + 'a>;
