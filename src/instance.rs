use crate::{
    class::QjClass,
    string::QjCString,
    tags::{
        QjAnyTag, QjBigDecimalTag, QjBigFloatTag, QjBigIntTag, QjBoolTag, QjFloat64Tag, QjIntTag, QjNullTag,
        QjObjectTag, QjReferenceTag, QjStringTag, QjSymbolTag, QjUndefinedTag, QjValueTag, QjVariant,
    },
    QjContext, QjRuntime,
};
use quilt::{ffi, Context, Value};
use std::{ffi::c_void, fmt, marker::PhantomData, ptr, ptr::null_mut, sync::atomic};

static DEBUG_GLOBAL_COUNT: atomic::AtomicU16 = atomic::AtomicU16::new(0);

macro_rules! call_with_context {
    ($self: expr, $name: ident) => {
        $self.value.$name($self.context)
    };
}

/// `Qj` is a value holder with a context.
pub struct Qj<'q, T> {
    value: Value<'q>,
    context: Context<'q>,
    _debug_count: u16,
    _type: PhantomData<T>,
}

impl<'q, T> Qj<'q, T> {
    pub(crate) fn from(value: Value<'q>, context: Context<'q>) -> Qj<'q, T> {
        let count = DEBUG_GLOBAL_COUNT.fetch_add(1, atomic::Ordering::SeqCst);
        let qj = Qj {
            value,
            context,
            _debug_count: count,
            _type: PhantomData,
        };
        qj.debug_trace("from");
        qj
    }

    #[inline]
    pub(crate) fn transmute<X>(&self) -> &Qj<'q, X> {
        unsafe { std::mem::transmute(self) }
    }

    // property

    #[inline]
    pub(crate) fn as_value(&self) -> Value<'q> {
        self.value
    }

    #[inline]
    pub(crate) fn context(&self) -> Context<'q> {
        self.context
    }

    // memory

    #[inline]
    pub(crate) unsafe fn free(this: &Self) {
        this.context.free_value(this.value);
        this.debug_trace("freed");
    }

    #[inline]
    pub(crate) fn dup(this: &Self) {
        this.context.dup_value(this.value);
        this.debug_trace("dup");
    }

    #[inline]
    pub(crate) fn detach(self) {
        Self::dup(&self)
    }

    fn debug_trace(&self, name: &str) {
        if let Some(rc) = self.value.ref_count() {
            log::trace!("{}: {:?} (rc: {})", name, self, rc);
        } else {
            log::trace!("{}: {:?} (value)", name, self);
        }
    }

    // type

    #[inline]
    pub fn is_null(&self) -> bool {
        self.value.is_null()
    }

    #[inline]
    #[doc(hidden)]
    pub fn is_exception(&self) -> bool {
        self.value.is_exception()
    }

    #[inline]
    pub fn is_undefined(&self) -> bool {
        self.value.is_undefined()
    }

    #[inline]
    pub fn is_uninitialized(&self) -> bool {
        self.value.is_uninitialized()
    }

    // conversion

    pub(crate) fn to_var(&self) -> QjVariant<'_> {
        match self.value.tag() {
            ffi::JS_TAG_BIG_DECIMAL => QjVariant::BigDecimal(self.transmute::<QjBigDecimalTag>().clone()),
            ffi::JS_TAG_BIG_INT => QjVariant::BigInt(self.transmute::<QjBigIntTag>().clone()),
            ffi::JS_TAG_BIG_FLOAT => QjVariant::BigFloat(self.transmute::<QjBigFloatTag>().clone()),
            ffi::JS_TAG_SYMBOL => QjVariant::Symbol(self.transmute::<QjSymbolTag>().clone()),
            ffi::JS_TAG_STRING => QjVariant::String(self.transmute::<QjStringTag>().clone()),
            ffi::JS_TAG_OBJECT => QjVariant::Object(self.transmute::<QjObjectTag>().clone()),
            ffi::JS_TAG_INT => QjVariant::Int(self.to_i32().unwrap()),
            ffi::JS_TAG_BOOL => QjVariant::Bool(self.to_bool().unwrap()),
            ffi::JS_TAG_NULL => QjVariant::Null,
            ffi::JS_TAG_UNDEFINED => QjVariant::Undefined,
            ffi::JS_TAG_UNINITIALIZED => QjVariant::Uninitialized,
            ffi::JS_TAG_CATCH_OFFSET => QjVariant::CatchOffset,
            ffi::JS_TAG_EXCEPTION => QjVariant::Exception,
            ffi::JS_TAG_FLOAT64 => QjVariant::Float64(self.to_f64().unwrap()),
            _ => panic!("invalid state"),
        }
    }

    #[inline]
    pub fn to_bool(&self) -> Option<bool> {
        call_with_context!(self, to_bool)
    }

    #[inline]
    pub fn to_i32(&self) -> Option<i32> {
        call_with_context!(self, to_i32)
    }

    #[inline]
    pub fn to_i64(&self) -> Option<i64> {
        call_with_context!(self, to_i64)
    }

    #[inline]
    pub fn to_f64(&self) -> Option<f64> {
        call_with_context!(self, to_f64)
    }

    #[inline]
    pub fn to_c_string(&self) -> Option<QjCString> {
        self.value
            .to_c_string(self.context)
            .map(|v| QjCString::from(v, self.context))
    }

    #[inline]
    pub fn to_string(&self) -> Option<String> {
        self.to_c_string().and_then(|v| v.to_string())
    }

    // object

    #[inline]
    pub fn get<K>(&self, key: K) -> Qj<'q, QjAnyTag>
    where
        K: AsRef<str>,
    {
        Qj::<QjAnyTag>::from(self.value.property_str(self.context, key), self.context)
    }

    #[inline]
    pub fn set<K, V>(&self, key: K, val: V)
    where
        K: AsRef<str>,
        V: AsRef<Qj<'q, QjAnyTag>>,
    {
        let val = val.as_ref();
        Qj::dup(val);
        self.value.set_property_str(self.context, key, val.as_value())
    }

    // class

    #[inline]
    pub(crate) fn get_opaque<C: QjClass + 'static>(&self) -> Option<&C> {
        let rt = QjRuntime::from(self.context.runtime());
        let clz = rt.get_class_id::<C>()?;
        let p = self.value.opaque(clz) as *const C;
        if p.is_null() {
            return None;
        }
        Some(unsafe { &*p })
    }

    #[inline]
    pub(crate) fn take_opaque<C: QjClass + 'static>(&mut self) -> Option<C> {
        let rt = QjRuntime::from(self.context.runtime());
        let clz = rt.get_class_id::<C>()?;
        let p = unsafe { self.value.opaque(clz) as *const C };
        if p.is_null() {
            return None;
        }
        self.value.set_opaque(null_mut());
        Some(unsafe { ptr::read(p) })
    }

    #[inline]
    pub fn set_opaque<C: QjClass + 'static>(&mut self, mut v: C) {
        let mut rt = QjRuntime::from(self.context.runtime());
        let _clz = rt.get_or_register_class_id::<C>();
        self.value.set_opaque(&mut v as *mut C as *mut c_void);
    }
}

impl<T> Drop for Qj<'_, T> {
    fn drop(&mut self) {
        unsafe { Self::free(self) }
    }
}

impl<T> Clone for Qj<'_, T> {
    fn clone(&self) -> Self {
        let qj = Qj {
            value: self.value,
            context: self.context,
            _debug_count: self._debug_count,
            _type: PhantomData,
        };
        Qj::dup(&qj);
        qj
    }
}

impl<T> fmt::Debug for Qj<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.write_str(format!("Qj({}, {:?})", self._debug_count, self.value).as_str())
    }
}

/// `QjVec` is a value vector with a context.
pub struct QjVec<'q, T> {
    values: Vec<Value<'q>>,
    context: Context<'q>,
    _debug_count: u16,
    _type: PhantomData<T>,
}

impl<'q, T> QjVec<'q, T> {
    pub(crate) fn from(values: &[Value<'q>], context: Context<'q>) -> QjVec<'q, T> {
        let count = DEBUG_GLOBAL_COUNT.fetch_add(1, atomic::Ordering::SeqCst);
        let qjs = QjVec {
            values: values.to_vec(),
            context,
            _debug_count: count,
            _type: PhantomData,
        };
        qjs.debug_trace("from");
        qjs
    }

    pub fn empty(ctx: QjContext<'q>) -> QjVec<'q, T> {
        Self::from(&[], ctx.into())
    }

    pub fn from_ref_slice(qjs: &[&Qj<'q, T>], ctx: QjContext<'q>) -> Option<QjVec<'q, T>> {
        let ctx = ctx.into();
        let mut vec = Vec::with_capacity(qjs.len());
        for qj in qjs {
            if qj.context != ctx {
                return None;
            }
            vec.push(qj.value);
            Qj::dup(qj)
        }
        Some(Self::from(vec.as_slice(), ctx))
    }

    pub fn from_slice(qjs: &[Qj<'q, T>], ctx: QjContext<'q>) -> Option<QjVec<'q, T>> {
        let vec: Vec<&Qj<'q, T>> = qjs.iter().map(|v| v).collect();
        Self::from_ref_slice(vec.as_slice(), ctx)
    }

    // property

    #[inline]
    pub(crate) fn as_vec(&self) -> &Vec<Value<'q>> {
        &self.values
    }

    #[inline]
    pub(crate) fn as_slice(&self) -> &[Value<'q>] {
        self.values.as_slice()
    }

    #[inline]
    pub(crate) fn context(&self) -> Context<'q> {
        self.context
    }

    // memory

    #[inline]
    pub(crate) unsafe fn free(this: &Self) {
        for value in &this.values {
            this.context.free_value(*value)
        }
        this.debug_trace("free");
    }

    #[inline]
    pub(crate) fn dup(this: &Self) {
        for value in &this.values {
            this.context.dup_value(*value)
        }
        this.debug_trace("dup");
    }

    #[inline]
    pub(crate) fn detach(self) {
        Self::dup(&self)
    }

    fn debug_trace(&self, name: &str) {
        log::debug!("{}: {:?} (Vec)", name, self);
    }

    // elements

    pub fn get(&self, idx: usize) -> Qj<'q, T> {
        let qj = Qj::<T>::from(self.values[idx], self.context);
        Qj::dup(&qj);
        qj
    }
}

impl<T> Drop for QjVec<'_, T> {
    fn drop(&mut self) {
        unsafe { Self::free(self) }
    }
}

impl<T> Clone for QjVec<'_, T> {
    fn clone(&self) -> Self {
        let qjs = QjVec {
            values: self.values.clone(),
            context: self.context,
            _debug_count: self._debug_count,
            _type: PhantomData,
        };
        QjVec::dup(&qjs);
        qjs
    }
}

impl<'q, T> Into<Vec<Qj<'q, T>>> for QjVec<'q, T> {
    fn into(self) -> Vec<Qj<'q, T>> {
        QjVec::dup(&self);
        self.values.iter().map(|v| Qj::<T>::from(*v, self.context)).collect()
    }
}

impl<T> fmt::Debug for QjVec<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.write_str(format!("QjVec({}, {:?})", self._debug_count, self.values).as_str())
    }
}

macro_rules! qj_type_map {
    ($from: ty: $($to: ty),*) => {
        impl<'q> ::std::convert::AsRef<$crate::Qj<'q, $from>> for $crate::Qj<'q, $from> {
            #[inline]
            fn as_ref(&self) -> &Qj<'q, $from> {
                self.transmute()
            }
        }

        $(
        impl<'q> ::std::convert::AsRef<$crate::Qj<'q, $to>> for $crate::Qj<'q, $from> {
            #[inline]
            fn as_ref(&self) -> &Qj<'q, $to> {
                self.transmute()
            }
        }

        impl<'q> ::std::convert::From<$crate::Qj<'q, $from>> for $crate::Qj<'q, $to> {
            #[inline]
            fn from(x: Qj<'q, $from>) -> Self {
                $crate::Qj::dup(&x);
                $crate::Qj::<$to>::from(x.as_value(), x.context())
            }
        }
        )*
    };
}

qj_type_map!(QjAnyTag: );

qj_type_map!(QjValueTag: QjAnyTag);
qj_type_map!(QjReferenceTag: QjAnyTag);

qj_type_map!(QjStringTag: QjAnyTag, QjReferenceTag);
qj_type_map!(QjObjectTag: QjAnyTag, QjReferenceTag);

qj_type_map!(QjIntTag: QjAnyTag, QjValueTag);
qj_type_map!(QjBoolTag: QjAnyTag, QjValueTag);
qj_type_map!(QjNullTag: QjAnyTag, QjValueTag);
qj_type_map!(QjUndefinedTag: QjAnyTag, QjValueTag);
qj_type_map!(QjFloat64Tag: QjAnyTag, QjValueTag);
