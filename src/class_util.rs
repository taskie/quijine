use crate::{
    class::{Class, ClassMethods},
    Context, Data, Object, Result, Runtime,
};
use log::trace;
use qjncore as qc;
use std::{ffi::CString, panic::UnwindSafe};

unsafe fn finalize<T: Class + 'static>(rrt: qc::Runtime, val: qc::Value) {
    let rt = Runtime::from(rrt);
    let clz = if let Some(clz) = rt.get_class_id::<T>() {
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

impl<'q, T: Class + 'static> ClassMethods<'q, T> for Methods<'q> {
    fn add_method<F, R>(&mut self, name: &str, method: F) -> Result<Object<'q>>
    where
        F: Fn(Context<'q>, &mut T, Data<'q>, &[Data<'q>]) -> Result<R> + UnwindSafe + Send + 'static,
        R: Into<Data<'q>> + 'q,
    {
        let ctx = self.context;
        let f = ctx.new_function(
            move |ctx, this, args| {
                let mut cloned = this.clone();
                let t = cloned.get_opaque_mut::<T>().unwrap();
                (method)(ctx, t, this, args)
            },
            name,
            0,
        )?;
        trace!("registering method: {}::{} ({:?})", T::name(), name, f);
        self.proto.set(name, f.clone())?;
        Ok(f)
    }
}

pub(crate) fn register_class<T: Class + 'static>(rctx: qc::Context, clz: qc::ClassId) -> Result<Object> {
    trace!("registering class: {} ({:?})", T::name(), clz);
    let ctx = Context::from(rctx);
    let mut rt = ctx.runtime();
    unsafe extern "C" fn finalizer<T: Class + 'static>(rt: *mut qjncore::raw::JSRuntime, val: qjncore::raw::JSValue) {
        let rt = qc::Runtime::from_ptr(rt);
        let val = qc::Value::from_raw_with_runtime(val, rt);
        finalize::<T>(rt, val)
    }
    if let Some(_class_def) = rt.get_class_def(clz) {
        // nop
    } else {
        // per Runtime
        let class_def = qc::ClassDef {
            class_name: CString::new(T::name()).unwrap(),
            finalizer: Some(finalizer::<T>),
            ..Default::default()
        };
        rt.register_class_def(clz, class_def);
        let class_def = rt.get_class_def(clz).unwrap();
        rt.new_class(clz, class_def)
    };
    // per Context
    let proto = ctx.new_object()?;
    Data::dup(&proto);
    rctx.set_class_proto(clz, proto.as_value());
    let mut methods = Methods {
        context: ctx,
        proto: &proto,
    };
    T::add_methods(&mut methods)?;
    T::setup_proto(ctx, &proto)?;
    Ok(proto)
}
