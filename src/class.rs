use crate::{
    convert::{FromQj, FromQjMulti, IntoQj},
    data::Data,
    types::Object,
    Context, Result, Runtime,
};
use log::trace;
use qjncore as qc;
use std::{cell::RefCell, ffi::CString, rc::Rc, sync::Arc};

#[derive(Debug)]
pub enum ClassObjectOpaqueInner<C: Class> {
    Box(*mut C),
    Rc(*const RefCell<C>),
    Arc(*const RefCell<C>),
}

#[derive(Debug)]
pub struct ClassObjectOpaque<C: Class>(ClassObjectOpaqueInner<C>);

impl<C: Class> ClassObjectOpaque<C> {
    pub(crate) fn with_box(v: Box<C>) -> Self {
        ClassObjectOpaque(ClassObjectOpaqueInner::Box(Box::into_raw(v)))
    }

    pub(crate) fn with_rc(v: Rc<RefCell<C>>) -> Self {
        ClassObjectOpaque(ClassObjectOpaqueInner::Rc(Rc::into_raw(v)))
    }

    pub(crate) fn with_arc(v: Arc<RefCell<C>>) -> Self {
        ClassObjectOpaque(ClassObjectOpaqueInner::Arc(Arc::into_raw(v)))
    }

    unsafe fn drop(this: &mut Self) {
        match this.0 {
            ClassObjectOpaqueInner::Box(v) => {
                Box::from_raw(v);
            }
            ClassObjectOpaqueInner::Rc(v) => {
                Rc::from_raw(v);
            }
            ClassObjectOpaqueInner::Arc(v) => {
                Arc::from_raw(v);
            }
        }
    }

    pub fn with<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&C) -> R,
    {
        match self.0 {
            ClassObjectOpaqueInner::Box(v) => f(unsafe { &*v }),
            ClassObjectOpaqueInner::Rc(v) => f(&unsafe { &*v }.borrow()),
            ClassObjectOpaqueInner::Arc(v) => f(&unsafe { &*v }.borrow()),
        }
    }

    pub fn with_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut C) -> R,
    {
        match self.0 {
            ClassObjectOpaqueInner::Box(v) => f(unsafe { &mut *v }),
            ClassObjectOpaqueInner::Rc(v) => f(&mut unsafe { &*v }.borrow_mut()),
            ClassObjectOpaqueInner::Arc(v) => f(&mut unsafe { &*v }.borrow_mut()),
        }
    }
}

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
    let p = val.opaque(clz) as *mut ClassObjectOpaque<T>;
    if p.is_null() {
        return;
    }
    let mut b = Box::from_raw(p);
    ClassObjectOpaque::drop(b.as_mut())
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
                let t = unsafe { cloned.opaque_mut::<C>() }.unwrap();
                t.with_mut(|t| (method)(ctx, t, T::from_qj(this)?, args))
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
