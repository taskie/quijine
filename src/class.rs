use crate::{
    convert::{FromQj, FromQjMulti, IntoQj},
    types::{ClassObject, Object},
    value::Value,
    Context, PropFlags, Result, Runtime,
};
use log::trace;
use quijine_core::{self as qc, raw};
use std::{ffi::CString, ptr::null_mut};

pub trait ClassProperties<'q, C: Class> {
    fn define_method<F, A, R>(&mut self, name: &str, method: F, length: i32) -> Result<Object<'q>>
    where
        F: Fn(&C, Context<'q>, ClassObject<'q, C>, A) -> Result<R> + 'static,
        A: FromQjMulti<'q>,
        R: IntoQj<'q> + 'q;
    fn define_method_mut<F, A, R>(&mut self, name: &str, method: F, length: i32) -> Result<Object<'q>>
    where
        F: Fn(&mut C, Context<'q>, ClassObject<'q, C>, A) -> Result<R> + 'static,
        A: FromQjMulti<'q>,
        R: IntoQj<'q> + 'q;
    fn define_get_set_mut<G, R1, S, A, R2>(
        &mut self,
        name: &str,
        getter: G,
        setter: S,
    ) -> Result<(Object<'q>, Object<'q>)>
    where
        G: Fn(&C, Context<'q>, ClassObject<'q, C>) -> Result<R1> + 'static,
        R1: IntoQj<'q> + 'q,
        S: Fn(&mut C, Context<'q>, ClassObject<'q, C>, A) -> Result<R2> + 'static,
        A: FromQj<'q>,
        R2: IntoQj<'q> + 'q;
    fn define_get_mut_set_mut<G, R1, S, A, R2>(
        &mut self,
        name: &str,
        getter: G,
        setter: S,
    ) -> Result<(Object<'q>, Object<'q>)>
    where
        G: Fn(&mut C, Context<'q>, ClassObject<'q, C>) -> Result<R1> + 'static,
        R1: IntoQj<'q> + 'q,
        S: Fn(&mut C, Context<'q>, ClassObject<'q, C>, A) -> Result<R2> + 'static,
        A: FromQj<'q>,
        R2: IntoQj<'q> + 'q;
    fn define_get<G, R>(&mut self, name: &str, getter: G) -> Result<Object<'q>>
    where
        G: Fn(&C, Context<'q>, ClassObject<'q, C>) -> Result<R> + 'static,
        R: IntoQj<'q> + 'q;
    fn define_get_mut<G, R>(&mut self, name: &str, getter: G) -> Result<Object<'q>>
    where
        G: Fn(&mut C, Context<'q>, ClassObject<'q, C>) -> Result<R> + 'static,
        R: IntoQj<'q> + 'q;
    fn define_set_mut<S, A, R>(&mut self, name: &str, setter: S) -> Result<Object<'q>>
    where
        S: Fn(&mut C, Context<'q>, ClassObject<'q, C>, A) -> Result<R> + 'static,
        A: FromQj<'q>,
        R: IntoQj<'q> + 'q;
}

#[allow(unused_variables)]
pub trait Class: Sized {
    fn name() -> &'static str;
    fn constructor(&mut self, ctx: Context, this: Value, args: &[Value]) -> Result<()> {
        Ok(())
    }
    fn constructor_length() -> i32 {
        0
    }
    fn define_properties<'q, P: ClassProperties<'q, Self>>(properties: &mut P) -> Result<()> {
        Ok(())
    }
    fn setup_proto<'q>(ctx: Context<'q>, proto: Object<'q>) -> Result<()> {
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
    // this Box was created by Value::set_opaque
    let _b = Box::from_raw(p);
}

struct Properties<'q> {
    proto: &'q Value<'q>,
    context: Context<'q>,
}

impl<'q, C: Class + 'static> ClassProperties<'q, C> for Properties<'q> {
    #[inline]
    fn define_method<F, A, R>(&mut self, name: &str, method: F, length: i32) -> Result<Object<'q>>
    where
        F: Fn(&C, Context<'q>, ClassObject<'q, C>, A) -> Result<R> + 'static,
        A: FromQjMulti<'q>,
        R: IntoQj<'q> + 'q,
    {
        self.define_method_mut(name, move |v, ctx, this, args| method(v, ctx, this, args), length)
    }

    fn define_method_mut<F, A, R>(&mut self, name: &str, method: F, length: i32) -> Result<Object<'q>>
    where
        F: Fn(&mut C, Context<'q>, ClassObject<'q, C>, A) -> Result<R> + 'static,
        A: FromQjMulti<'q>,
        R: IntoQj<'q> + 'q,
    {
        let ctx = self.context;
        let f = ctx.new_function_from(
            move |ctx, this: Value<'q>, args| {
                let mut cloned = this.clone();
                let v = cloned.opaque_mut::<C>().unwrap();
                (method)(v, ctx, unsafe { Value::copy_unchecked(this) }, args)
            },
            name,
            length,
        )?;
        trace!("registering method: {}::{} ({:?})", C::name(), name, f);
        self.proto
            .define_property_value_from(name, f.clone(), PropFlags::CONFIGURABLE | PropFlags::WRITABLE)?;
        Ok(f)
    }

    #[inline]
    fn define_get_set_mut<G, R1, S, A, R2>(
        &mut self,
        name: &str,
        getter: G,
        setter: S,
    ) -> Result<(Object<'q>, Object<'q>)>
    where
        G: Fn(&C, Context<'q>, ClassObject<'q, C>) -> Result<R1> + 'static,
        R1: IntoQj<'q> + 'q,
        S: Fn(&mut C, Context<'q>, ClassObject<'q, C>, A) -> Result<R2> + 'static,
        A: FromQj<'q>,
        R2: IntoQj<'q> + 'q,
    {
        self.define_get_mut_set_mut(name, move |v, ctx, this| getter(v, ctx, this), setter)
    }

    fn define_get_mut_set_mut<G, R1, S, A, R2>(
        &mut self,
        name: &str,
        getter: G,
        setter: S,
    ) -> Result<(Object<'q>, Object<'q>)>
    where
        G: Fn(&mut C, Context<'q>, ClassObject<'q, C>) -> Result<R1> + 'static,
        R1: IntoQj<'q> + 'q,
        S: Fn(&mut C, Context<'q>, ClassObject<'q, C>, A) -> Result<R2> + 'static,
        A: FromQj<'q>,
        R2: IntoQj<'q> + 'q,
    {
        let ctx = self.context;
        let g = make_getter(ctx, getter)?;
        let s = make_setter(ctx, setter)?;
        trace!("registering get/set: {}::{} ({:?}, {:?})", C::name(), name, g, s);
        self.proto.define_property_get_set_from(
            name,
            g.clone(),
            s.clone(),
            PropFlags::CONFIGURABLE | PropFlags::ENUMERABLE,
        )?;
        Ok((g, s))
    }

    #[inline]
    fn define_get<G, R>(&mut self, name: &str, getter: G) -> Result<Object<'q>>
    where
        G: Fn(&C, Context<'q>, ClassObject<'q, C>) -> Result<R> + 'static,
        R: IntoQj<'q> + 'q,
    {
        self.define_get_mut(name, move |v, ctx, this| getter(v, ctx, this))
    }

    fn define_get_mut<G, R>(&mut self, name: &str, getter: G) -> Result<Object<'q>>
    where
        G: Fn(&mut C, Context<'q>, ClassObject<'q, C>) -> Result<R> + 'static,
        R: IntoQj<'q> + 'q,
    {
        let ctx = self.context;
        let g = make_getter(ctx, getter)?;
        trace!("registering get: {}::{} ({:?})", C::name(), name, g);
        self.proto.define_property_get_set_from(
            name,
            g.clone(),
            ctx.undefined(),
            PropFlags::CONFIGURABLE | PropFlags::ENUMERABLE,
        )?;
        Ok(g)
    }

    fn define_set_mut<S, A, R>(&mut self, name: &str, setter: S) -> Result<Object<'q>>
    where
        S: Fn(&mut C, Context<'q>, ClassObject<'q, C>, A) -> Result<R> + 'static,
        A: FromQj<'q>,
        R: IntoQj<'q> + 'q,
    {
        let ctx = self.context;
        let s = make_setter(ctx, setter)?;
        trace!("registering set: {}::{} ({:?})", C::name(), name, s);
        self.proto.define_property_get_set_from(
            name,
            ctx.undefined(),
            s.clone(),
            PropFlags::CONFIGURABLE | PropFlags::ENUMERABLE,
        )?;
        Ok(s)
    }
}

fn make_getter<'q, C, G, R>(ctx: Context<'q>, getter: G) -> Result<Object<'q>>
where
    C: Class + 'static,
    G: Fn(&mut C, Context<'q>, ClassObject<'q, C>) -> Result<R> + 'static,
    R: IntoQj<'q> + 'q,
{
    ctx.new_function_from(
        move |ctx, this: Value<'q>, _args: Vec<Value<'q>>| {
            let mut cloned = this.clone();
            let v = cloned.opaque_mut::<C>().unwrap();
            (getter)(v, ctx, unsafe { Value::copy_unchecked(this) })
        },
        "get",
        0,
    )
}

fn make_setter<'q, C, S, A, R>(ctx: Context<'q>, setter: S) -> Result<Object<'q>>
where
    C: Class + 'static,
    S: Fn(&mut C, Context<'q>, ClassObject<'q, C>, A) -> Result<R> + 'static,
    A: FromQj<'q>,
    R: IntoQj<'q> + 'q,
{
    ctx.new_function_from(
        move |ctx, this: Value<'q>, args: Vec<Value<'q>>| {
            let arg = args.get(0).cloned().unwrap_or_else(|| ctx.undefined().into());
            let mut cloned = this.clone();
            let v = cloned.opaque_mut::<C>().unwrap();
            (setter)(v, ctx, unsafe { Value::copy_unchecked(this) }, A::from_qj(arg)?)
        },
        "set",
        1,
    )
}

pub(crate) fn register_class<C: Class + 'static>(rctx: qc::Context, clz: qc::ClassId) -> Result<Object> {
    trace!("registering class: {} ({:?})", C::name(), clz);
    let ctx = Context::from_raw(rctx);
    let mut rt = ctx.runtime();
    unsafe extern "C" fn finalizer<C: Class + 'static>(rt: *mut raw::JSRuntime, val: raw::JSValue) {
        let rt = qc::Runtime::from_raw(rt);
        let val = qc::Value::from_raw_with_runtime(val, rt);
        finalize::<C>(rt, val)
    }
    if let Some(_class_def) = rt.class_def(clz) {
        // nop
    } else {
        // per Runtime
        let class_name = CString::new(C::name()).unwrap();
        rt.register_class_name(class_name.clone());
        let class_def = unsafe {
            qc::ClassDef::from_raw(raw::JSClassDef {
                class_name: rt.class_name(&class_name).unwrap().as_ptr(),
                finalizer: Some(finalizer::<C>),
                gc_mark: None,
                call: None,
                exotic: null_mut(),
            })
        };
        rt.register_class_def(clz, class_def);
        let class_def = rt.class_def(clz).unwrap();
        rt.new_class(clz, class_def)
    };
    // per Context
    let proto = ctx.new_object()?;
    Value::dup(&proto);
    rctx.set_class_proto(clz, *proto.as_raw());
    let mut properties = Properties {
        context: ctx,
        proto: &proto,
    };
    C::define_properties(&mut properties)?;
    C::setup_proto(ctx, proto.clone())?;
    Ok(proto)
}
