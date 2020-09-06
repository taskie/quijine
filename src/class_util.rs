use crate::{
    class::{QjClass, QjClassMethods},
    tags::{QjAnyTag, QjObjectTag},
    Qj, QjContext, QjResult, QjRuntime, QjVec,
};
use log::trace;
use quilt::{self, ffi, ClassDef, ClassId, Context, Runtime, Value};
use std::{marker::Sync, ptr};

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
    let _ = ptr::read(p);
}

struct Methods<'q> {
    proto: &'q Qj<'q, QjObjectTag>,
    context: QjContext<'q>,
    function_list: Vec<ffi::JSCFunctionListEntry>,
}

impl<'q, T: QjClass + 'static> QjClassMethods<'q, T> for Methods<'q> {
    fn add_method<F>(&mut self, name: &str, method: F)
    where
        F: 'static
            + Send
            + Fn(QjContext<'q>, &T, Qj<'q, QjAnyTag>, QjVec<'q, QjAnyTag>) -> QjResult<'q, Qj<'q, QjAnyTag>>
            + Sync,
    {
        let ctx = self.context;
        let f = ctx.new_function(
            move |ctx, this, args| {
                let cloned = this.clone();
                let t = cloned.get_opaque::<T>().unwrap();
                (method)(ctx, t, this, args)
            },
            name,
            0,
        );
        trace!("registering method: {}::{} ({:?})", T::name(), name, f);
        self.proto.set(name, f);
    }
}

pub(crate) fn register_class<T: QjClass + 'static>(rt: Runtime, clz: ClassId) {
    trace!("registering class: {} ({:?})", T::name(), clz);
    let rctx = Context::new(rt);
    let ctx = QjContext::from(rctx);
    unsafe extern "C" fn finalizer<T: QjClass + 'static>(rt: *mut quilt::ffi::JSRuntime, val: quilt::ffi::JSValue) {
        let rt = Runtime::from_ptr(rt);
        let val = Value::from_raw_with_runtime(val, rt);
        finalize::<T>(rt, val)
    }
    rt.new_class(
        clz,
        &ClassDef {
            class_name: T::name().to_owned(),
            finalizer: Some(finalizer::<T>),
            ..Default::default()
        },
    );
    let proto = ctx.new_object();
    rctx.set_class_proto(clz, proto.as_value());
    proto.as_value().set_property_function_list(rctx, &[]);
    let mut methods = Methods {
        context: ctx,
        proto: &proto,
        function_list: vec![],
    };
    T::add_methods(&mut methods);
    T::setup_proto(ctx, &proto);
    proto.detach();
    unsafe { Context::free(rctx) };
}

#[cfg(test)]
mod tests {
    use crate::{
        class::{QjClass, QjClassMethods},
        run_with_context,
        tags::QjObjectTag,
        Qj, QjContext, QjEvalFlags, QjResult,
    };

    struct S1 {
        name: String,
        pos: (i32, i32),
    }

    impl QjClass for S1 {
        fn name() -> &'static str {
            "S1"
        }

        fn add_methods<'q, T: QjClassMethods<'q, Self>>(methods: &mut T) {
            methods.add_method("name", |ctx, t, _this, _args| {
                Ok(ctx.new_string(t.name.as_str()).into())
            });
        }
    }

    #[test]
    fn test_opaque() {
        env_logger::init();
        run_with_context(|ctx| {
            let global = ctx.global_object();
            global.set(
                "S1",
                ctx.new_function(
                    |ctx, _this, _args| {
                        let mut obj = ctx.new_object_class::<S1>();
                        let s1 = S1 {
                            name: "hoge".to_owned(),
                            pos: (3, 4),
                        };
                        obj.set_opaque(s1);
                        Ok(obj.into())
                    },
                    "S1",
                    0,
                ),
            );
            let result = ctx
                .eval("var s1 = S1(); s1", "<input>", QjEvalFlags::TYPE_GLOBAL)
                .unwrap();
            println!("{:?}", result.to_var());
            let result = ctx
                .eval("Object.getPrototypeOf({})", "<input>", QjEvalFlags::TYPE_GLOBAL)
                .unwrap();
            println!("{:?}", result.to_var());
            let result = ctx
                .eval("Object.getPrototypeOf(s1)", "<input>", QjEvalFlags::TYPE_GLOBAL)
                .unwrap();
            println!("{:?}", result.to_var());
        })
    }
}
