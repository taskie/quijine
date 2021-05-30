use crate::{class::QjClass, string::QjCString, tags::QjVariant, types::String as QjString, QjRuntime};
use qjncore::{Context, Value, ValueTag};
use std::{convert::TryInto, ffi::c_void, fmt, intrinsics::transmute, mem, sync::atomic};

static DEBUG_GLOBAL_COUNT: atomic::AtomicU16 = atomic::AtomicU16::new(0);

macro_rules! call_with_context {
    ($self: expr, $name: ident) => {
        $self.value.$name($self.context)
    };
}

/// `Data` is a value holder with a context.
pub struct Data<'q> {
    value: Value<'q>,
    context: Context<'q>,
    _debug_count: u16,
}

impl<'q> Data<'q> {
    pub(crate) fn from(value: Value<'q>, context: Context<'q>) -> Data<'q> {
        let count = DEBUG_GLOBAL_COUNT.fetch_add(1, atomic::Ordering::SeqCst);
        let qj = Data {
            value,
            context,
            _debug_count: count,
        };
        qj.debug_trace("from");
        qj
    }

    // property

    #[inline]
    pub(crate) fn as_value(&self) -> Value<'q> {
        self.value
    }

    // force conversion

    #[inline]
    pub(crate) fn as_data(&self) -> &Data<'q> {
        self
    }

    /// # Safety
    /// Must be type-safe in JavaScript.
    #[inline]
    pub(crate) unsafe fn as_any<T: Into<Data<'q>>>(&self) -> &T {
        transmute(self)
    }

    // memory

    #[inline]
    pub(crate) unsafe fn free(this: &Self) {
        this.context.free_value(this.value);
        this.debug_trace("free");
    }

    #[inline]
    pub(crate) fn dup(this: &Self) {
        this.context.dup_value(this.value);
        this.debug_trace("dup");
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

    #[inline]
    pub(crate) fn tag(&self) -> ValueTag {
        self.value.tag()
    }

    pub fn to_variant(&self) -> QjVariant<'_> {
        match self.tag() {
            ValueTag::BigDecimal => QjVariant::BigDecimal(self.clone().try_into().unwrap()),
            ValueTag::BigInt => QjVariant::BigInt(self.clone().try_into().unwrap()),
            ValueTag::BigFloat => QjVariant::BigFloat(self.clone().try_into().unwrap()),
            ValueTag::Symbol => QjVariant::Symbol(self.clone().try_into().unwrap()),
            ValueTag::String => QjVariant::String(TryInto::<QjString>::try_into(self.clone()).unwrap()),
            ValueTag::Object => QjVariant::Object(self.clone().try_into().unwrap()),
            ValueTag::Int => QjVariant::Int(self.to_i32().unwrap()),
            ValueTag::Bool => QjVariant::Bool(self.to_bool().unwrap()),
            ValueTag::Null => QjVariant::Null,
            ValueTag::Undefined => QjVariant::Undefined,
            ValueTag::Uninitialized => QjVariant::Uninitialized,
            ValueTag::CatchOffset => QjVariant::CatchOffset,
            ValueTag::Exception => QjVariant::Exception,
            ValueTag::Float64 => QjVariant::Float64(self.to_f64().unwrap()),
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

    #[inline]
    pub(crate) fn to_ptr(&self) -> Option<*mut c_void> {
        self.value.ptr()
    }

    // object

    #[inline]
    pub fn get<K>(&self, key: K) -> Data<'q>
    where
        K: AsRef<str>,
    {
        Data::from(self.value.property_str(self.context, key), self.context)
    }

    #[inline]
    pub fn set<K, V>(&self, key: K, val: V)
    where
        K: AsRef<str>,
        V: AsRef<Data<'q>>,
    {
        let val = val.as_ref();
        Data::dup(val);
        self.value.set_property_str(self.context, key, val.as_value())
    }

    // class

    #[inline]
    pub fn prototype(&self) -> Data<'q> {
        Data::from(self.value.prototype(self.context), self.context)
    }

    #[inline]
    pub(crate) fn get_opaque_mut<C: QjClass + 'static>(&mut self) -> Option<&mut C> {
        let rt = QjRuntime::from(self.context.runtime());
        let clz = rt.get_class_id::<C>()?;
        let p = self.value.opaque(clz) as *mut C;
        if p.is_null() {
            return None;
        }
        Some(unsafe { &mut *p })
    }

    #[inline]
    pub fn set_opaque<C: QjClass + 'static>(&mut self, mut v: Box<C>) {
        let mut rt = QjRuntime::from(self.context.runtime());
        let _clz = rt.get_or_register_class_id::<C>();
        unsafe { self.value.set_opaque(v.as_mut() as *mut C as *mut c_void) };
        mem::forget(v);
    }
}

impl Drop for Data<'_> {
    fn drop(&mut self) {
        unsafe { Self::free(self) }
    }
}

impl Clone for Data<'_> {
    fn clone(&self) -> Self {
        let qj = Data {
            value: self.value,
            context: self.context,
            _debug_count: self._debug_count,
        };
        Data::dup(&qj);
        qj
    }
}

impl fmt::Debug for Data<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.write_str(format!("Qj({}, {:?})", self._debug_count, self.value).as_str())
    }
}

impl<'q> AsRef<Data<'q>> for Data<'q> {
    fn as_ref(&self) -> &Data<'q> {
        self.as_data()
    }
}
