use crate::{
    class::{register_class, Class},
    convert::{AsData, FromQj, FromQjMulti, IntoQj, IntoQjMulti},
    error::{ErrorValue, Result},
    runtime::Runtime,
    types::{Bool, Float64, Int, Null, Object, String as QjString, Undefined},
    Data, EvalFlags, RuntimeScope,
};
use qjncore::{self as qc, raw, AsJsValue};
use std::{any::TypeId, collections::HashSet, ffi::c_void, fmt, os::raw::c_int};

pub struct ContextOpaque {
    registered_classes: HashSet<TypeId>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Context<'q>(qc::Context<'q>);

impl<'q> Context<'q> {
    #[inline]
    pub(crate) fn from_raw(ctx: qc::Context<'q>) -> Self {
        Context(ctx)
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
    unsafe fn wrap_result<T: AsData<'q>>(self, val: qc::Value<'q>) -> Result<T> {
        Data::wrap_result(val, self.0)
    }

    #[inline]
    pub fn eval(self, code: &str, filename: &str, eval_flags: EvalFlags) -> Result<Data<'q>> {
        unsafe { self.wrap_result(self.0.eval(code, filename, eval_flags)) }
    }

    #[inline]
    pub fn eval_into<R: FromQj<'q>>(self, code: &str, filename: &str, eval_flags: EvalFlags) -> Result<R> {
        R::from_qj(self.eval(code, filename, eval_flags)?)
    }

    #[inline]
    pub fn eval_function(self, func_obj: Data<'q>) -> Result<Data<'q>> {
        Data::dup(&func_obj);
        unsafe { self.wrap_result(self.0.eval_function(func_obj.as_value())) }
    }

    #[inline]
    pub fn eval_function_into<F: IntoQj<'q>, R: FromQj<'q>>(self, func_obj: F) -> Result<R> {
        R::from_qj(self.eval_function(func_obj.into_qj(self)?)?)
    }

    #[inline]
    pub fn call(self, func_obj: Data<'q>, this_obj: Data<'q>, args: &[Data<'q>]) -> Result<Data<'q>> {
        let qc_args: Vec<_> = args.iter().map(|v| v.as_value()).collect();
        let val = self.0.call(func_obj.as_value(), this_obj.as_value(), qc_args);
        unsafe { self.wrap_result(val) }
    }

    #[inline]
    pub fn call_into<F, T, A, R>(self, func_obj: F, this_obj: T, args: A) -> Result<R>
    where
        F: IntoQj<'q>,
        T: IntoQj<'q>,
        A: IntoQjMulti<'q>,
        R: FromQj<'q>,
    {
        let qj_args = args.into_qj_multi(self)?;
        R::from_qj(self.call(func_obj.into_qj(self)?, this_obj.into_qj(self)?, qj_args.as_ref())?)
    }

    #[inline]
    pub fn call_into_void<F, T, A>(self, func_obj: F, this_obj: T, args: A) -> Result<()>
    where
        F: IntoQj<'q>,
        T: IntoQj<'q>,
        A: IntoQjMulti<'q>,
    {
        let qj_args = args.into_qj_multi(self)?;
        self.call(func_obj.into_qj(self)?, this_obj.into_qj(self)?, qj_args.as_ref())?;
        Ok(())
    }

    #[inline]
    pub fn global_object(self) -> Result<Object<'q>> {
        unsafe { self.wrap_result(self.0.global_object()) }
    }

    #[inline]
    pub fn new_object(self) -> Result<Object<'q>> {
        unsafe { self.wrap_result(self.0.new_object()) }
    }

    #[inline]
    pub(crate) fn new_object_class<C: Class + 'static>(mut self) -> Result<Object<'q>> {
        let clz = self.register_class::<C>()?;
        unsafe { self.wrap_result(self.0.new_object_class(clz)) }
    }

    #[inline]
    pub fn new_object_with_opaque<C: Class + 'static>(self, v: Box<C>) -> Result<Object<'q>> {
        let mut obj = self.new_object_class::<C>()?;
        obj.set_opaque(v);
        Ok(obj)
    }

    unsafe fn new_value<T: AsData<'q>>(self, v: qc::Value<'q>) -> T {
        Data::from_raw_parts(v, self.0).into_unchecked()
    }

    #[inline]
    pub fn new_bool(self, v: bool) -> Bool<'q> {
        unsafe { self.new_value(self.0.new_bool(v)) }
    }

    #[inline]
    pub fn new_int32(self, v: i32) -> Int<'q> {
        unsafe { self.new_value(self.0.new_int32(v)) }
    }

    #[inline]
    pub fn new_int64(self, v: i64) -> Data<'q> {
        unsafe { self.new_value(self.0.new_int64(v)) }
    }

    #[inline]
    pub fn new_float64(self, v: f64) -> Float64<'q> {
        unsafe { self.new_value(self.0.new_float64(v)) }
    }

    #[inline]
    pub fn new_string(self, v: &str) -> Result<QjString<'q>> {
        unsafe { self.wrap_result(self.0.new_string(v)) }
    }

    // callback

    #[inline]
    pub fn new_function<F>(self, func: F, name: &str, length: i32) -> Result<Object<'q>>
    where
        F: Fn(Context<'q>, Data<'q>, &'q [Data<'q>]) -> Result<Data<'q>> + Send + 'q,
    {
        self.new_callback(Box::new(func), name, length)
    }

    #[inline]
    pub fn new_function_with<F, T, A, R>(self, func: F, name: &str, length: i32) -> Result<Object<'q>>
    where
        F: Fn(Context<'q>, T, A) -> Result<R> + Send + 'q,
        T: FromQj<'q>,
        A: FromQjMulti<'q, 'q>,
        R: IntoQj<'q> + 'q,
    {
        self.new_function(
            move |ctx, this, args| func(ctx, T::from_qj(this)?, A::from_qj_multi(args)?)?.into_qj(ctx),
            name,
            length,
        )
    }

    #[inline]
    pub(crate) fn new_callback<R>(self, func: Box<Callback<'q, 'q, R>>, _name: &str, length: i32) -> Result<Object<'q>>
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
            let ctx = qc::Context::from_raw(ctx);
            let this = qc::Value::from_raw(js_this, ctx);
            let mut args: Vec<qc::Value> = Vec::with_capacity(argc as usize);
            for i in 0..argc {
                let p = argv.offset(i as isize);
                let any = qc::Value::from_raw(*p, ctx);
                args.push(any);
            }
            let cb = qc::Value::from_raw(*func_data, ctx);
            log::debug!("load pointer from ArrayBuffer");
            let func = cb.array_buffer_to_sized::<Box<Callback<R>>>(ctx).unwrap();

            log::debug!("this");
            let this = Data::from_raw_parts(this, ctx);
            Data::dup(&this);
            log::debug!("args");
            let args: Vec<_> = args.iter().map(|v| Data::from_raw_parts(*v, ctx)).collect();
            args.iter().for_each(Data::dup);
            let ctx = Context::from_raw(ctx);

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
                        ErrorValue::JsError(_) => qc::Value::undefined(), // use original Error
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
            let _buf: Object = self.wrap_result(cb)?;
            log::debug!("new c function data");
            let cfd = self.0.new_c_function_data(Some(call::<R>), length, 0, vec![cb]);
            self.wrap_result(cfd)
        }
    }

    // special values

    pub fn undefined(self) -> Undefined<'q> {
        unsafe { Data::from_raw_parts(qc::Value::undefined(), self.0).into_unchecked() }
    }

    pub fn null(self) -> Null<'q> {
        unsafe { Data::from_raw_parts(qc::Value::null(), self.0).into_unchecked() }
    }

    // json

    pub fn parse_json(self, buf: &str, filename: &str) -> Result<Data<'q>> {
        unsafe { self.wrap_result(self.0.parse_json(buf, filename)) }
    }

    pub fn parse_json_into<R: FromQj<'q>>(self, buf: &str, filename: &str) -> Result<R> {
        R::from_qj(self.parse_json(buf, filename)?)
    }

    pub fn json_stringify(self, obj: Data<'q>, replacer: Data<'q>, space0: Data<'q>) -> Result<QjString<'q>> {
        unsafe {
            self.wrap_result::<QjString>(self.0.json_stringify(
                obj.into_qj(self)?.as_value(),
                replacer.into_qj(self)?.as_value(),
                space0.into_qj(self)?.as_value(),
            ))
        }
    }

    pub fn json_stringify_into<O, P, S, R>(self, obj: O, replacer: P, space0: S) -> Result<R>
    where
        O: IntoQj<'q>,
        P: IntoQj<'q>,
        S: IntoQj<'q>,
        R: FromQj<'q>,
    {
        R::from_qj(
            self.json_stringify(obj.into_qj(self)?, replacer.into_qj(self)?, space0.into_qj(self)?)?
                .into(),
        )
    }

    // class

    pub(crate) fn register_class<T: Class + 'static>(&mut self) -> Result<qc::ClassId> {
        let type_id = TypeId::of::<T>();
        let class_id = self.runtime().get_or_register_class_id::<T>();
        if self.opaque().registered_classes.contains(&type_id) {
            return Ok(class_id);
        }
        register_class::<T>(self.0, class_id)?;
        self.opaque_mut().registered_classes.insert(type_id);
        Ok(class_id)
    }
}

pub struct ContextScope<'r>(Context<'r>);

impl<'r> ContextScope<'r> {
    pub fn new(rt: Runtime) -> ContextScope {
        let ctx = qc::Context::new(rt.into());
        let opaque = Box::new(ContextOpaque {
            registered_classes: HashSet::new(),
        });
        ctx.set_opaque(Box::into_raw(opaque) as *mut c_void);
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
            qc::Context::free(self.0 .0)
        }
    }
}

impl fmt::Debug for ContextScope<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::result::Result<(), fmt::Error> {
        f.write_str(format!("ContextScope({:?})", self.0).as_str())
    }
}

pub(crate) type Callback<'q, 'a, R> = dyn Fn(Context<'q>, Data<'q>, &'a [Data<'q>]) -> Result<R> + 'a;
