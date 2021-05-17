use crate::{
    class::{QjClass, QjClassMethods},
    tags::{QjAnyTag, QjObjectTag},
    Qj, QjContext, QjResult, QjRuntime, QjVec,
};
use log::trace;
use qjncore::{self, ClassDef, ClassId, Context, Runtime, Value};
use std::{ffi::CString, marker::Sync};

unsafe fn finalize<T: QjClass + 'static>(rrt: Runtime, val: Value) {
    let rt = QjRuntime::from(rrt);
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
    proto: &'q Qj<'q, QjObjectTag>,
    context: QjContext<'q>,
}

impl<'q, T: QjClass + 'static> QjClassMethods<'q, T> for Methods<'q> {
    fn add_method<F>(&mut self, name: &str, method: F)
    where
        F: 'static
            + Send
            + Fn(QjContext<'q>, &mut T, Qj<'q, QjAnyTag>, QjVec<'q, QjAnyTag>) -> QjResult<'q, Qj<'q, QjAnyTag>>
            + Sync,
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
        );
        trace!("registering method: {}::{} ({:?})", T::name(), name, f);
        self.proto.set(name, f);
    }
}

pub(crate) fn register_class<T: QjClass + 'static>(rctx: Context, clz: ClassId) {
    trace!("registering class: {} ({:?})", T::name(), clz);
    let ctx = QjContext::from(rctx);
    let mut rt = ctx.runtime();
    unsafe extern "C" fn finalizer<T: QjClass + 'static>(rt: *mut qjncore::ffi::JSRuntime, val: qjncore::ffi::JSValue) {
        let rt = Runtime::from_ptr(rt);
        let val = Value::from_raw_with_runtime(val, rt);
        finalize::<T>(rt, val)
    }
    if let Some(_class_def) = rt.get_class_def(clz) {
        // nop
    } else {
        // per Runtime
        let class_def = ClassDef {
            class_name: CString::new(T::name()).unwrap(),
            finalizer: Some(finalizer::<T>),
            ..Default::default()
        };
        rt.register_class_def(clz, class_def);
        let class_def = rt.get_class_def(clz).unwrap();
        rt.new_class(clz, class_def)
    };
    // per Context
    let proto = ctx.new_object();
    Qj::dup(&proto);
    rctx.set_class_proto(clz, proto.as_value());
    let mut methods = Methods {
        context: ctx,
        proto: &proto,
    };
    T::add_methods(&mut methods);
    T::setup_proto(ctx, &proto);
}

#[cfg(test)]
mod tests {}
