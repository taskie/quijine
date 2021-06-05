use crate::{
    convert::{FromQj, FromQjMulti, IntoQj},
    data::Data,
    types::Object,
    Context, Result, Runtime,
};
use log::trace;
use qjncore as qc;
use std::ffi::CString;

pub trait ClassMethods<'q, C: Class> {
    fn add_method<F, T, A, R>(&mut self, name: &str, method: F) -> Result<Object<'q>>
    where
        F: Fn(Context<'q>, &mut C, T, A) -> Result<R> + Send + 'static,
        T: FromQj<'q>,
        A: FromQjMulti<'q, 'q>,
        R: IntoQj<'q> + 'q;
}

pub trait Class: Sized + Send {
    fn name() -> &'static str;
    fn add_methods<'q, T: ClassMethods<'q, Self>>(_methods: &mut T) -> Result<()> {
        Ok(())
    }
    fn setup_proto(_ctx: Context, _proto: &Object) -> Result<()> {
        Ok(())
    }
}

unsafe fn finalize<T: Class + 'static>(rrt: qc::Runtime, val: qc::Value) {
    let rt = Runtime::from(rrt);
    let clz = if let Some(clz) = rt.class_id::<T>() {
        clz
    } else {
        return;
    };
    let p = val.opaque(clz) as *mut T;
    if p.is_null() {
        return;
    }
    let _b = Box::from_raw(p);
}

struct Methods<'q> {
    proto: &'q Data<'q>,
    context: Context<'q>,
}

impl<'q, C: Class + 'static> ClassMethods<'q, C> for Methods<'q> {
    fn add_method<F, T, A, R>(&mut self, name: &str, method: F) -> Result<Object<'q>>
    where
        F: Fn(Context<'q>, &mut C, T, A) -> Result<R> + Send + 'static,
        T: FromQj<'q>,
        A: FromQjMulti<'q, 'q>,
        R: IntoQj<'q> + 'q,
    {
        let ctx = self.context;
        let f = ctx.new_function_with(
            move |ctx, this: Data<'q>, args| {
                let mut cloned = this.clone();
                let t = cloned.opaque_mut::<C>().unwrap();
                (method)(ctx, t, T::from_qj(this)?, args)
            },
            name,
            0,
        )?;
        trace!("registering method: {}::{} ({:?})", C::name(), name, f);
        self.proto.set(name, f.clone())?;
        Ok(f)
    }
}

pub(crate) fn register_class<T: Class + 'static>(rctx: qc::Context, clz: qc::ClassId) -> Result<Object> {
    trace!("registering class: {} ({:?})", T::name(), clz);
    let ctx = Context::from_raw(rctx);
    let mut rt = ctx.runtime();
    unsafe extern "C" fn finalizer<T: Class + 'static>(rt: *mut qjncore::raw::JSRuntime, val: qjncore::raw::JSValue) {
        let rt = qc::Runtime::from_raw(rt);
        let val = qc::Value::from_raw_with_runtime(val, rt);
        finalize::<T>(rt, val)
    }
    if let Some(_class_def) = rt.class_def(clz) {
        // nop
    } else {
        // per Runtime
        let class_def = qc::ClassDef {
            class_name: CString::new(T::name()).unwrap(),
            finalizer: Some(finalizer::<T>),
            ..Default::default()
        };
        rt.register_class_def(clz, class_def);
        let class_def = rt.class_def(clz).unwrap();
        rt.new_class(clz, class_def)
    };
    // per Context
    let proto = ctx.new_object()?;
    Data::dup(&proto);
    rctx.set_class_proto(clz, *proto.as_raw());
    let mut methods = Methods {
        context: ctx,
        proto: &proto,
    };
    T::add_methods(&mut methods)?;
    T::setup_proto(ctx, &proto)?;
    Ok(proto)
}
