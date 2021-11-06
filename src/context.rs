use crate::{
    atom::Atom,
    class::{register_class, Class},
    convert::{FromQj, FromQjMulti, IntoQj, IntoQjMulti},
    error::ErrorValue,
    result::Result,
    runtime::Runtime,
    types::{Bool, ClassObject, Float64, Int, Null, Object, String as QjString, Undefined},
    Error, ErrorKind, EvalFlags, Exception, ModuleDef, PropFlags, RuntimeScope, Value,
};
use qc::{ReadObjFlags, WriteObjFlags};
use quijine_core::{self as qc, raw, AsJsValue};
use std::{
    any::TypeId, collections::HashSet, convert::TryInto, ffi::c_void, fmt, os::raw::c_int, result::Result as StdResult,
};

macro_rules! def_throw_error {
    ($name: ident) => {
        #[inline]
        pub fn $name(self, message: &str) -> Exception<'q> {
            Value::from_raw_parts(self.0.$name(message), self.0)
                .try_into()
                .unwrap()
        }
    };
}

pub struct ContextOpaque {
    registered_classes: HashSet<TypeId>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Context<'q>(qc::Context<'q>);

impl<'q> Context<'q> {
    def_throw_error!(throw_syntax_error);

    def_throw_error!(throw_type_error);

    def_throw_error!(throw_reference_error);

    def_throw_error!(throw_range_error);

    def_throw_error!(throw_internal_error);

    #[inline]
    pub(crate) fn from_raw(ctx: qc::Context<'q>) -> Self {
        Context(ctx)
    }

    #[inline]
    pub(crate) fn as_raw(self) -> qc::Context<'q> {
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
    pub(crate) fn class_proto<C: Class + 'static>(self) -> Result<Value<'q>> {
        let class_id = self.runtime().get_or_register_class_id::<C>();
        Ok(Value::from_raw_parts(self.0.class_proto(class_id), self.0))
    }

    #[inline]
    pub(crate) fn map_err_to_exception<T, E>(self, val: StdResult<T, E>) -> Result<T> {
        if let Ok(v) = val {
            Ok(v)
        } else {
            Err(self.internal_js_error())
        }
    }

    #[inline]
    pub(crate) unsafe fn wrap_result<T: AsRef<Value<'q>>>(self, val: qc::Value<'q>) -> Result<T> {
        if val.is_exception() {
            Err(self.internal_js_error())
        } else {
            Ok(Value::from_raw_parts(val, self.0).into_unchecked())
        }
    }

    #[inline]
    unsafe fn wrap_result_atom(self, val: qc::Atom<'q>) -> Result<Atom> {
        if val.is_null() {
            Err(Error::with_str(ErrorKind::InternalError, "null atom"))
        } else {
            Ok(Atom::from_raw_parts(val, self.0))
        }
    }

    #[inline]
    pub(crate) fn internal_js_error(self) -> Error {
        Error::from_js_error(ErrorKind::InternalError, self.take_exception())
    }

    #[inline]
    pub fn eval(self, code: &str, filename: &str, eval_flags: EvalFlags) -> Result<Value<'q>> {
        unsafe { self.wrap_result(self.0.eval(code, filename, eval_flags)) }
    }

    #[inline]
    pub fn eval_into<R: FromQj<'q>>(self, code: &str, filename: &str, eval_flags: EvalFlags) -> Result<R> {
        R::from_qj(self.eval(code, filename, eval_flags)?)
    }

    #[inline]
    fn eval_function_raw(self, func_obj: Value<'q>) -> Result<Value<'q>> {
        Value::dup(&func_obj);
        unsafe { self.wrap_result(self.0.eval_function(*func_obj.as_raw())) }
    }

    #[inline]
    pub fn eval_function<F: Into<Value<'q>>>(self, func_obj: F) -> Result<Value<'q>> {
        self.eval_function_raw(func_obj.into_qj(self)?)
    }

    #[inline]
    fn call_raw(self, func_obj: Value<'q>, this_obj: Value<'q>, args: &[Value<'q>]) -> Result<Value<'q>> {
        let qc_args: Vec<_> = args.iter().map(|v| *v.as_raw()).collect();
        let val = self.0.call(*func_obj.as_raw(), *this_obj.as_raw(), &qc_args);
        unsafe { self.wrap_result(val) }
    }

    #[inline]
    pub fn call<F, T, A>(self, func_obj: F, this_obj: T, args: A) -> Result<Value<'q>>
    where
        F: Into<Value<'q>>,
        T: IntoQj<'q>,
        A: IntoQjMulti<'q>,
    {
        self.call_raw(
            func_obj.into_qj(self)?,
            this_obj.into_qj(self)?,
            args.into_qj_multi(self)?.as_ref(),
        )
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
    pub fn new_global_constructor<C: Class + Default + 'static>(self) -> Result<Object<'q>> {
        let ctor = self.new_class_constructor::<C>()?;
        self.global_object()?.define_property_value(
            C::name(),
            ctor.clone(),
            PropFlags::WRITABLE | PropFlags::CONFIGURABLE,
        )?;
        Ok(ctor)
    }

    #[inline]
    pub fn new_global_constructor_with_new<C, F>(self, f: F) -> Result<Object<'q>>
    where
        C: Class + 'static,
        F: Fn(Context<'q>, Value<'q>, &[Value<'q>]) -> C + 'q,
    {
        let ctor = self.new_class_constructor_with_new::<C, F>(f)?;
        self.global_object()?.define_property_value(
            C::name(),
            ctor.clone(),
            PropFlags::WRITABLE | PropFlags::CONFIGURABLE,
        )?;
        Ok(ctor)
    }

    #[inline]
    pub fn new_class_constructor<C: Class + Default + 'static>(self) -> Result<Object<'q>> {
        self.new_class_constructor_with_new(|_, _, _| C::default())
    }

    #[inline]
    pub fn new_class_constructor_with_new<C, F>(mut self, f: F) -> Result<Object<'q>>
    where
        C: Class + 'static,
        F: Fn(Context<'q>, Value<'q>, &[Value<'q>]) -> C + 'q,
    {
        self.register_class::<C>()?;
        let f = self.new_function_raw(
            move |ctx, this, args| {
                let mut obj = ctx.new_object_with_opaque(f(ctx, this.clone(), args))?;
                C::constructor(&mut obj.opaque_mut().unwrap(), ctx, this, args)?;
                Ok(obj.into())
            },
            C::name(),
            C::constructor_length(),
        )?;
        f.set_constructor_bit(true)?;
        f.set_constructor(self.class_proto::<C>()?)?;
        Ok(f)
    }

    #[inline]
    pub(crate) fn new_object_class<C: Class + 'static>(mut self) -> Result<Object<'q>> {
        let clz = self.register_class::<C>()?;
        unsafe { self.wrap_result(self.0.new_object_class(clz)) }
    }

    #[inline]
    pub fn new_object_with_opaque<C: Class + 'static>(self, v: C) -> Result<ClassObject<'q, C>> {
        let mut obj = self.new_object_class::<C>()?;
        obj.set_opaque(v);
        Ok(unsafe { Value::copy_unchecked(obj) })
    }

    #[inline]
    pub fn new_array(self) -> Result<Object<'q>> {
        unsafe { self.wrap_result(self.0.new_array()) }
    }

    unsafe fn new_value<T: AsRef<Value<'q>>>(self, v: qc::Value<'q>) -> T {
        Value::from_raw_parts(v, self.0).into_unchecked()
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
    pub fn new_int64(self, v: i64) -> Value<'q> {
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

    // atom

    #[inline]
    pub fn new_atom(self, s: &str) -> Result<Atom<'q>> {
        unsafe { self.wrap_result_atom(self.0.new_atom(s)) }
    }

    #[inline]
    pub fn new_atom_with_u32(self, n: u32) -> Result<Atom<'q>> {
        unsafe { self.wrap_result_atom(self.0.new_atom_u_int32(n)) }
    }

    #[inline]
    pub(crate) fn atom_to_value(self, atom: &Atom<'q>) -> Result<Value<'q>> {
        unsafe { self.wrap_result(self.0.atom_to_value(*atom.as_raw())) }
    }

    #[inline]
    pub(crate) fn atom_to_string(self, atom: &Atom<'q>) -> Result<QjString<'q>> {
        unsafe { self.wrap_result(self.0.atom_to_string(*atom.as_raw())) }
    }

    #[inline]
    pub(crate) fn value_to_atom(self, v: &Value<'q>) -> Result<Atom<'q>> {
        unsafe { self.wrap_result_atom(self.0.value_to_atom(*v.as_raw())) }
    }

    // callback

    #[inline]
    fn new_function_raw<F>(self, func: F, name: &str, length: i32) -> Result<Object<'q>>
    where
        F: Fn(Context<'q>, Value<'q>, &'q [Value<'q>]) -> Result<Value<'q>> + 'q,
    {
        self.new_callback(Box::new(func), name, length)
    }

    #[inline]
    pub fn new_function<F, R>(self, func: F, name: &str, length: i32) -> Result<Object<'q>>
    where
        F: Fn(Context<'q>, Value<'q>, &'q [Value<'q>]) -> Result<R> + 'q,
        R: IntoQj<'q> + 'q,
    {
        self.new_function_raw(move |ctx, this, args| func(ctx, this, args)?.into_qj(ctx), name, length)
    }

    #[inline]
    pub fn new_function_from<F, T, A, R>(self, func: F, name: &str, length: i32) -> Result<Object<'q>>
    where
        F: Fn(Context<'q>, T, A) -> Result<R> + 'q,
        T: FromQj<'q>,
        A: FromQjMulti<'q>,
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
        R: Into<Value<'q>> + 'q,
    {
        unsafe extern "C" fn call<'q, R: Into<Value<'q>>>(
            ctx: *mut raw::JSContext,
            js_this: raw::JSValue,
            argc: c_int,
            argv: *mut raw::JSValue,
            _magic: c_int,
            func_value: *mut raw::JSValue,
        ) -> raw::JSValue {
            let ctx = qc::Context::from_raw(ctx);
            let this = qc::Value::from_raw(js_this, ctx);
            let mut args: Vec<qc::Value> = Vec::with_capacity(argc as usize);
            for i in 0..argc {
                let p = argv.offset(i as isize);
                let any = qc::Value::from_raw(*p, ctx);
                args.push(any);
            }
            let cb = qc::Value::from_raw(*func_value, ctx);
            log::debug!("load pointer from ArrayBuffer");
            let func = cb.array_buffer_as_ref::<Box<Callback<R>>>(ctx).unwrap();

            log::debug!("this");
            let this = Value::from_raw_parts(this, ctx);
            Value::dup(&this);
            log::debug!("args");
            let args: Vec<_> = args.iter().map(|v| Value::from_raw_parts(*v, ctx)).collect();
            args.iter().for_each(Value::dup);
            let ctx = Context::from_raw(ctx);

            log::debug!("invoke start");
            let r = (**func)(ctx, this, args.as_slice());
            let res = match r {
                Ok(t) => {
                    let t = t.into();
                    Value::dup(&t);
                    t.as_raw().as_js_value()
                }
                Err(e) => {
                    let v = e.value;
                    match v {
                        ErrorValue::None => {
                            ctx.throw_internal_error("some error occured");
                        }
                        ErrorValue::String(s) => {
                            ctx.throw_internal_error(&s);
                        }
                        ErrorValue::JsError(_) => (), // use original Error
                        ErrorValue::External(e) => {
                            ctx.throw_internal_error(&format!("{}", e));
                        }
                    };
                    qc::Value::exception().as_js_value()
                }
            };
            log::debug!("invoke end");
            res
        }
        unsafe {
            log::debug!("save pointer to ArrayBuffer");
            // Box of Sized
            let func_box = Box::new(func);
            let cb = self.0.new_array_buffer_from_boxed(func_box);
            let _cb: Object = self.wrap_result(cb)?; // check errors
            log::debug!("new c function data");
            let cfd = self.0.new_c_function_data(Some(call::<R>), length, 0, &[cb]);
            self.wrap_result(cfd)
        }
    }

    // special values

    pub fn undefined(self) -> Undefined<'q> {
        unsafe { Value::from_raw_parts(qc::Value::undefined(), self.0).into_unchecked() }
    }

    pub fn null(self) -> Null<'q> {
        unsafe { Value::from_raw_parts(qc::Value::null(), self.0).into_unchecked() }
    }

    // exception

    /// Returns the pending exception (cannot be called twice).
    #[inline]
    pub fn take_exception(self) -> Value<'q> {
        Value::from_raw_parts(self.0.exception(), self.0)
    }

    #[inline]
    pub fn throw(self, obj: Value<'q>) -> Exception<'q> {
        Value::dup(&obj);
        let value = self.0.throw(*obj.as_raw());
        Value::from_raw_parts(value, self.0).try_into().unwrap()
    }

    #[inline]
    pub fn reset_uncacheable_error(self) {
        self.0.reset_uncacheable_error()
    }

    #[inline]
    pub fn new_error(self) -> Result<Object<'q>> {
        unsafe { self.wrap_result(self.0.new_error()) }
    }

    #[inline]
    pub fn throw_out_of_memory(self) -> Exception<'q> {
        Value::from_raw_parts(self.0.throw_out_of_memory(), self.0)
            .try_into()
            .unwrap()
    }

    // json

    pub fn parse_json(self, buf: &str, filename: &str) -> Result<Value<'q>> {
        unsafe { self.wrap_result(self.0.parse_json(buf, filename)) }
    }

    fn json_stringify_raw(self, obj: Value<'q>, replacer: Value<'q>, space0: Value<'q>) -> Result<QjString<'q>> {
        unsafe {
            self.wrap_result::<QjString>(
                self.0
                    .json_stringify(*obj.as_raw(), *replacer.as_raw(), *space0.as_raw()),
            )
        }
    }

    pub fn json_stringify<O, P, S>(self, obj: O, replacer: P, space0: S) -> Result<QjString<'q>>
    where
        O: Into<Value<'q>>,
        P: IntoQj<'q>,
        S: IntoQj<'q>,
    {
        self.json_stringify_raw(obj.into_qj(self)?, replacer.into_qj(self)?, space0.into_qj(self)?)
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

    // Module

    #[inline]
    pub fn new_c_module(self, name_str: &str, func: raw::JSModuleInitFunc) -> ModuleDef<'q> {
        unsafe { ModuleDef::from_raw_parts(self.0.new_c_module(name_str, func), self.0) }
    }

    // object writer/reader

    #[inline]
    pub fn write_object(self, obj: Value, flags: WriteObjFlags) -> Result<Vec<u8>> {
        self.0
            .write_object(*obj.as_raw(), flags)
            .ok_or_else(|| self.internal_js_error())
    }

    #[inline]
    pub fn read_object(self, buf: &[u8], flags: ReadObjFlags) -> Result<Value<'q>> {
        unsafe { self.wrap_result(self.0.read_object(buf, flags)) }
    }
}

pub struct ContextScope<'r>(Context<'r>);

impl<'r> ContextScope<'r> {
    fn new_internal(rt: Runtime, raw: bool) -> ContextScope {
        let ctx = if raw {
            qc::Context::new_raw(rt.into())
        } else {
            qc::Context::new(rt.into())
        };
        let opaque = Box::new(ContextOpaque {
            registered_classes: HashSet::new(),
        });
        ctx.set_opaque(Box::into_raw(opaque) as *mut c_void);
        ContextScope(Context(ctx))
    }

    pub fn new(rt: Runtime) -> ContextScope {
        Self::new_internal(rt, false)
    }

    pub fn new_raw(rt: Runtime) -> ContextScope {
        Self::new_internal(rt, true)
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
            // opaque must be bound until values in the context will be freed
            let _opaque = Box::from_raw((self.0).0.opaque() as *mut ContextOpaque);
            qc::Context::free(self.0 .0)
        }
    }
}

impl fmt::Debug for ContextScope<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::result::Result<(), fmt::Error> {
        f.write_str(format!("ContextScope({:?})", self.0).as_str())
    }
}

pub(crate) type Callback<'q, 'a, R> = dyn Fn(Context<'q>, Value<'q>, &'a [Value<'q>]) -> Result<R> + 'a;
