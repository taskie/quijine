use crate::{
    convert::{FromQj, FromQjMulti, IntoQj},
    data::Data,
    types::Object,
    Context, PropFlags, Result, Runtime,
};
use log::trace;
use quijine_core as qc;
use std::ffi::CString;

pub trait ClassMethods<'q, C: Class> {
    fn add_method<T, F, A, R>(&mut self, name: &str, method: F) -> Result<Object<'q>>
    where
        T: FromQj<'q>,
        F: Fn(&mut C, Context<'q>, T, A) -> Result<R> + 'static,
        A: FromQjMulti<'q, 'q>,
        R: IntoQj<'q> + 'q;
    fn add_get_set<T, G, R1, S, A, R2>(&mut self, name: &str, getter: G, setter: S) -> Result<(Object<'q>, Object<'q>)>
    where
        T: FromQj<'q>,
        G: Fn(&mut C, Context<'q>, T) -> Result<R1> + 'static,
        R1: IntoQj<'q> + 'q,
        S: Fn(&mut C, Context<'q>, T, A) -> Result<R2> + 'static,
        A: FromQj<'q>,
        R2: IntoQj<'q> + 'q;
    fn add_get<T, G, R>(&mut self, name: &str, getter: G) -> Result<Object<'q>>
    where
        T: FromQj<'q>,
        G: Fn(&mut C, Context<'q>, T) -> Result<R> + 'static,
        R: IntoQj<'q> + 'q;
    fn add_set<T, S, A, R>(&mut self, name: &str, setter: S) -> Result<Object<'q>>
    where
        T: FromQj<'q>,
        S: Fn(&mut C, Context<'q>, T, A) -> Result<R> + 'static,
        A: FromQj<'q>,
        R: IntoQj<'q> + 'q;
}

#[allow(unused_variables)]
pub trait Class: Sized {
    fn name() -> &'static str;
    fn constructor<'q>(&mut self, ctx: Context<'q>, this: Data, args: &[Data]) -> Result<()> {
        Ok(())
    }
    fn constructor_length() -> i32 {
        0
    }
    fn add_methods<'q, M: ClassMethods<'q, Self>>(methods: &mut M) -> Result<()> {
        Ok(())
    }
    fn setup_proto<'q>(ctx: Context<'q>, proto: &Object<'q>) -> Result<()> {
        Ok(())
    }
}

unsafe fn finalize<C: Class + 'static>(rrt: qc::Runtime, val: qc::Value) {
    let rt = Runtime::from(rrt);
    let clz = if let Some(clz) = rt.class_id::<C>() {
        clz
    } else {
        return;
    };
    let p = val.opaque(clz) as *mut C;
    if p.is_null() {
        return;
    }
    // this Box was created by Data::set_opaque
    let _b = Box::from_raw(p);
}

struct Methods<'q> {
    proto: &'q Data<'q>,
    context: Context<'q>,
}

impl<'q, C: Class + 'static> ClassMethods<'q, C> for Methods<'q> {
    fn add_method<T, F, A, R>(&mut self, name: &str, method: F) -> Result<Object<'q>>
    where
        T: FromQj<'q>,
        F: Fn(&mut C, Context<'q>, T, A) -> Result<R> + 'static,
        A: FromQjMulti<'q, 'q>,
        R: IntoQj<'q> + 'q,
    {
        let ctx = self.context;
        let f = ctx.new_function_with(
            move |ctx, this: Data<'q>, args| {
                let mut cloned = this.clone();
                let v = cloned.opaque_mut::<C>().unwrap();
                (method)(v, ctx, T::from_qj(this)?, args)
            },
            name,
            0,
        )?;
        trace!("registering method: {}::{} ({:?})", C::name(), name, f);
        self.proto
            .define_property_value_with(name, f.clone(), PropFlags::CONFIGURABLE | PropFlags::WRITABLE)?;
        Ok(f)
    }

    fn add_get_set<T, G, R1, S, A, R2>(&mut self, name: &str, getter: G, setter: S) -> Result<(Object<'q>, Object<'q>)>
    where
        T: FromQj<'q>,
        G: Fn(&mut C, Context<'q>, T) -> Result<R1> + 'static,
        R1: IntoQj<'q> + 'q,
        S: Fn(&mut C, Context<'q>, T, A) -> Result<R2> + 'static,
        A: FromQj<'q>,
        R2: IntoQj<'q> + 'q,
    {
        let ctx = self.context;
        let g = make_getter(ctx, getter)?;
        let s = make_setter(ctx, setter)?;
        trace!("registering get/set: {}::{} ({:?}, {:?})", C::name(), name, g, s);
        self.proto.define_property_get_set_with(
            name,
            g.clone(),
            s.clone(),
            PropFlags::CONFIGURABLE | PropFlags::ENUMERABLE,
        )?;
        Ok((g, s))
    }

    fn add_get<T, G, R>(&mut self, name: &str, getter: G) -> Result<Object<'q>>
    where
        T: FromQj<'q>,
        G: Fn(&mut C, Context<'q>, T) -> Result<R> + 'static,
        R: IntoQj<'q> + 'q,
    {
        let ctx = self.context;
        let g = make_getter(ctx, getter)?;
        trace!("registering get: {}::{} ({:?})", C::name(), name, g);
        self.proto.define_property_get_set_with(
            name,
            g.clone(),
            ctx.undefined(),
            PropFlags::CONFIGURABLE | PropFlags::ENUMERABLE,
        )?;
        Ok(g)
    }

    fn add_set<T, S, A, R>(&mut self, name: &str, setter: S) -> Result<Object<'q>>
    where
        T: FromQj<'q>,
        S: Fn(&mut C, Context<'q>, T, A) -> Result<R> + 'static,
        A: FromQj<'q>,
        R: IntoQj<'q> + 'q,
    {
        let ctx = self.context;
        let s = make_setter(ctx, setter)?;
        trace!("registering set: {}::{} ({:?})", C::name(), name, s);
        self.proto.define_property_get_set_with(
            name,
            ctx.undefined(),
            s.clone(),
            PropFlags::CONFIGURABLE | PropFlags::ENUMERABLE,
        )?;
        Ok(s)
    }
}

fn make_getter<'q, C, T, G, R>(ctx: Context<'q>, getter: G) -> Result<Object<'q>>
where
    C: Class + 'static,
    T: FromQj<'q>,
    G: Fn(&mut C, Context<'q>, T) -> Result<R> + 'static,
    R: IntoQj<'q> + 'q,
{
    ctx.new_function_with(
        move |ctx, this: Data<'q>, _args: &[Data]| {
            let mut cloned = this.clone();
            let v = cloned.opaque_mut::<C>().unwrap();
            (getter)(v, ctx, T::from_qj(this)?)
        },
        "get",
        0,
    )
}

fn make_setter<'q, C, T, S, A, R>(ctx: Context<'q>, setter: S) -> Result<Object<'q>>
where
    C: Class + 'static,
    T: FromQj<'q>,
    S: Fn(&mut C, Context<'q>, T, A) -> Result<R> + 'static,
    A: FromQj<'q>,
    R: IntoQj<'q> + 'q,
{
    ctx.new_function_with(
        move |ctx, this: Data<'q>, args: &[Data]| {
            let arg = args.get(0).cloned().unwrap_or_else(|| ctx.undefined().into());
            let mut cloned = this.clone();
            let v = cloned.opaque_mut::<C>().unwrap();
            (setter)(v, ctx, T::from_qj(this)?, A::from_qj(arg)?)
        },
        "set",
        1,
    )
}

pub(crate) fn register_class<C: Class + 'static>(rctx: qc::Context, clz: qc::ClassId) -> Result<Object> {
    trace!("registering class: {} ({:?})", C::name(), clz);
    let ctx = Context::from_raw(rctx);
    let mut rt = ctx.runtime();
    unsafe extern "C" fn finalizer<C: Class + 'static>(
        rt: *mut quijine_core::raw::JSRuntime,
        val: quijine_core::raw::JSValue,
    ) {
        let rt = qc::Runtime::from_raw(rt);
        let val = qc::Value::from_raw_with_runtime(val, rt);
        finalize::<C>(rt, val)
    }
    if let Some(_class_def) = rt.class_def(clz) {
        // nop
    } else {
        // per Runtime
        let class_def = qc::ClassDef {
            class_name: CString::new(C::name()).unwrap(),
            finalizer: Some(finalizer::<C>),
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
    C::add_methods(&mut methods)?;
    C::setup_proto(ctx, &proto)?;
    Ok(proto)
}
