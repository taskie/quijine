use crate::{
    class::Class,
    context::Context,
    convert::AsData,
    error::{Error, ErrorKind, Result},
    runtime::Runtime,
    string::CString as QjCString,
    types::{Tag, Variant},
};
use qjncore as qc;
use std::{
    convert::TryInto,
    ffi::c_void,
    fmt,
    mem::{self, forget, transmute_copy},
    result::Result as StdResult,
    sync::atomic,
};

static DEBUG_GLOBAL_COUNT: atomic::AtomicU16 = atomic::AtomicU16::new(0);

macro_rules! call_with_context {
    ($self: expr, $name: ident) => {
        $self.value.$name($self.context)
    };
}

/// `Data` is a value holder with a context.
pub struct Data<'q> {
    value: qc::Value<'q>,
    context: qc::Context<'q>,
    _debug_count: u16,
}

impl<'q> Data<'q> {
    pub(crate) fn from_raw_parts(value: qc::Value<'q>, context: qc::Context<'q>) -> Data<'q> {
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
    pub(crate) fn as_value(&self) -> qc::Value<'q> {
        self.value
    }

    #[inline]
    pub(crate) fn context(&self) -> Context<'q> {
        Context::from_raw(self.context)
    }

    // force conversion

    #[inline]
    pub(crate) fn as_data_raw(&self) -> &Data<'q> {
        self
    }

    /// # Safety
    /// Must be type-safe in JavaScript.
    #[inline]
    pub(crate) unsafe fn as_any<T: Into<Data<'q>>>(&self) -> &T {
        &*(self as *const Data as *const T)
    }

    #[inline]
    pub(crate) unsafe fn into_unchecked<T: AsData<'q>>(self) -> T {
        let ret = transmute_copy(&self);
        forget(self);
        ret
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
        if let Some(rc) = self.value.debug_ref_count() {
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
    pub(crate) fn tag(&self) -> Tag {
        self.value.tag()
    }

    pub fn to_variant(&self) -> Variant<'_> {
        match self.tag() {
            Tag::BigDecimal => Variant::BigDecimal(self.clone().try_into().unwrap()),
            Tag::BigInt => Variant::BigInt(self.clone().try_into().unwrap()),
            Tag::BigFloat => Variant::BigFloat(self.clone().try_into().unwrap()),
            Tag::Symbol => Variant::Symbol(self.clone().try_into().unwrap()),
            Tag::String => Variant::String(self.clone().try_into().unwrap()),
            Tag::Object => Variant::Object(self.clone().try_into().unwrap()),
            Tag::Int => Variant::Int(self.to_i32().unwrap()),
            Tag::Bool => Variant::Bool(self.to_bool().unwrap()),
            Tag::Null => Variant::Null,
            Tag::Undefined => Variant::Undefined,
            Tag::Uninitialized => Variant::Uninitialized,
            Tag::CatchOffset => Variant::CatchOffset,
            Tag::Exception => Variant::Exception,
            Tag::Float64 => Variant::Float64(self.to_f64().unwrap()),
        }
    }

    #[inline]
    fn ok_or_type_error<T>(&self, v: Option<T>) -> Result<T> {
        v.ok_or_else(|| Error::from_data(ErrorKind::TypeError, self.clone()))
    }

    #[inline]
    pub(crate) unsafe fn wrap_result<T: AsData<'q>>(val: qc::Value<'q>, ctx: qc::Context<'q>) -> Result<T> {
        if val.is_exception() {
            Err(Error::from_js_error(
                ErrorKind::InternalError,
                Data::from_raw_parts(ctx.exception(), ctx),
            ))
        } else {
            Ok(Data::from_raw_parts(val, ctx).into_unchecked())
        }
    }

    #[inline]
    pub fn to_bool(&self) -> Result<bool> {
        self.ok_or_type_error(call_with_context!(self, to_bool))
    }

    #[inline]
    pub fn to_i32(&self) -> Result<i32> {
        self.ok_or_type_error(call_with_context!(self, to_i32))
    }

    #[inline]
    pub fn to_i64(&self) -> Result<i64> {
        self.ok_or_type_error(call_with_context!(self, to_i64))
    }

    #[inline]
    pub fn to_f64(&self) -> Result<f64> {
        self.ok_or_type_error(call_with_context!(self, to_f64))
    }

    #[inline]
    pub fn to_c_string(&self) -> Result<QjCString> {
        self.ok_or_type_error(
            self.value
                .to_c_string(self.context)
                .map(|v| QjCString::from(v, self.context)),
        )
    }

    #[inline]
    pub fn to_string(&self) -> Result<String> {
        self.ok_or_type_error(
            self.value
                .to_c_string(self.context)
                .and_then(|v| v.to_str())
                .map(|v| v.to_owned()),
        )
    }

    #[inline]
    pub(crate) fn to_ptr(&self) -> Result<*mut c_void> {
        self.ok_or_type_error(self.value.ptr())
    }

    // object

    #[inline]
    pub fn get<K>(&self, key: K) -> Result<Data<'q>>
    where
        K: AsRef<str>,
    {
        unsafe { Data::wrap_result(self.value.property_str(self.context, key), self.context) }
    }

    #[inline]
    pub fn set<K, V>(&self, key: K, val: V) -> Result<bool>
    where
        K: AsRef<str>,
        V: AsRef<Data<'q>>,
    {
        let val = val.as_ref();
        Data::dup(val);
        let ret = self.value.set_property_str(self.context, key, val.as_value());
        ret.ok_or_else(|| {
            Error::from_js_error(
                ErrorKind::InternalError,
                Data::from_raw_parts(self.context.exception(), self.context),
            )
        })
    }

    // class

    #[inline]
    pub fn prototype(&self) -> Result<Data<'q>> {
        unsafe { Data::wrap_result(self.value.prototype(self.context), self.context) }
    }

    #[inline]
    pub(crate) fn opaque_mut<C: Class + 'static>(&mut self) -> Option<&mut C> {
        let rt = Runtime::from(self.context.runtime());
        let clz = rt.class_id::<C>()?;
        let p = self.value.opaque(clz) as *mut C;
        if p.is_null() {
            return None;
        }
        Some(unsafe { &mut *p })
    }

    #[inline]
    pub(crate) fn set_opaque<C: Class + 'static>(&mut self, mut v: Box<C>) {
        let mut rt = Runtime::from(self.context.runtime());
        let _clz = rt.get_or_register_class_id::<C>();
        self.value.set_opaque(v.as_mut() as *mut C as *mut c_void);
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> StdResult<(), fmt::Error> {
        f.write_str(format!("Qj({}, {:?})", self._debug_count, self.value).as_str())
    }
}

impl<'q> AsRef<Data<'q>> for Data<'q> {
    fn as_ref(&self) -> &Data<'q> {
        self.as_data()
    }
}
