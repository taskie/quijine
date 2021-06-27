use crate::{
    atom::{Atom, PropertyEnum},
    class::Class,
    context::Context,
    convert::{FromQj, IntoQj, IntoQjAtom},
    error::{Error, ErrorKind},
    result::Result,
    runtime::Runtime,
    string::CString as QjCString,
    types::{Tag, Variant},
};
use qc::{GPNFlags, PropFlags};
use quijine_core as qc;
#[cfg(feature = "debug_leak")]
use std::sync::atomic;
use std::{
    convert::TryInto,
    ffi::c_void,
    fmt,
    mem::{forget, transmute_copy},
    result::Result as StdResult,
};

#[cfg(feature = "debug_leak")]
static DEBUG_GLOBAL_COUNT: atomic::AtomicU16 = atomic::AtomicU16::new(0);

macro_rules! call_with_context {
    ($self: expr, $name: ident) => {
        $self.value.$name($self.context)
    };
}

/// `Value` is a value holder with a context.
pub struct Value<'q> {
    value: qc::Value<'q>,
    context: qc::Context<'q>,
    #[cfg(feature = "debug_leak")]
    _debug_count: u16,
}

impl<'q> Value<'q> {
    pub(crate) fn from_raw_parts(value: qc::Value<'q>, context: qc::Context<'q>) -> Value<'q> {
        #[cfg(feature = "debug_leak")]
        let count = DEBUG_GLOBAL_COUNT.fetch_add(1, atomic::Ordering::SeqCst);
        let qj = Value {
            value,
            context,
            #[cfg(feature = "debug_leak")]
            _debug_count: count,
        };
        qj.debug_trace("from");
        qj
    }

    // property

    #[inline]
    pub(crate) fn as_raw(&self) -> &qc::Value<'q> {
        &self.value
    }

    #[inline]
    pub fn context(&self) -> Context<'q> {
        Context::from_raw(self.context)
    }

    // force conversion

    #[inline]
    pub(crate) fn as_data_raw(&self) -> &Value<'q> {
        self
    }

    /// # Safety
    /// Must be type-safe in JavaScript.
    #[inline]
    pub(crate) unsafe fn as_any<T: Into<Value<'q>>>(&self) -> &T {
        &*(self as *const Value as *const T)
    }

    #[inline]
    pub(crate) unsafe fn copy_unchecked<T: AsRef<Value<'q>>, U: AsRef<Value<'q>>>(this: T) -> U {
        let ret = transmute_copy(&this);
        forget(this);
        ret
    }

    #[inline]
    pub(crate) unsafe fn into_unchecked<T: AsRef<Value<'q>>>(self) -> T {
        Self::copy_unchecked(self)
    }

    // memory

    #[inline]
    pub(crate) unsafe fn free(this: &Self) {
        this.debug_trace("free");
        this.context.free_value(this.value);
    }

    #[inline]
    pub(crate) fn dup(this: &Self) {
        this.context.dup_value(this.value);
        this.debug_trace("dup");
    }

    #[allow(unused_variables)]
    #[inline]
    fn debug_trace(&self, name: &str) {
        #[cfg(feature = "debug_leak")]
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

    #[inline]
    pub fn is_array(&self) -> bool {
        self.value.is_array(self.context)
    }

    #[inline]
    pub fn is_error(&self) -> bool {
        self.value.is_error(self.context)
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
            Tag::Module => Variant::Module(self.clone().try_into().unwrap()),
            Tag::FunctionBytecode => Variant::FunctionBytecode(self.clone().try_into().unwrap()),
            Tag::Object => Variant::Object(self.clone().try_into().unwrap()),
            Tag::Int => Variant::Int(self.to_i32().unwrap()),
            Tag::Bool => Variant::Bool(self.to_bool().unwrap()),
            Tag::Null => Variant::Null,
            Tag::Undefined => Variant::Undefined,
            Tag::Uninitialized => Variant::Uninitialized,
            Tag::CatchOffset => Variant::CatchOffset(Default::default()),
            Tag::Exception => Variant::Exception,
            Tag::Float64 => Variant::Float64(self.to_f64().unwrap()),
        }
    }

    #[inline]
    fn ok_or_type_error<T>(&self, v: Option<T>) -> Result<T> {
        v.ok_or_else(|| Error::from_data(ErrorKind::TypeError, self.clone()))
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
                .map(|v| QjCString::from_raw_parts(v, self.context)),
        )
    }

    #[inline]
    pub fn to_string(&self) -> Result<String> {
        self.to_c_string().and_then(|s| s.to_string())
    }

    #[inline]
    pub(crate) fn to_ptr(&self) -> Result<*mut c_void> {
        self.ok_or_type_error(self.value.ptr())
    }

    // atom

    pub fn to_atom(&self) -> Result<Atom<'q>> {
        self.context().data_to_atom(self)
    }

    // object

    #[inline]
    pub fn property(&self, key: Atom<'q>) -> Result<Value<'q>> {
        unsafe {
            self.context()
                .wrap_result(self.value.property(self.context, *key.as_raw()))
        }
    }

    #[inline]
    pub fn get<K, R>(&self, key: K) -> Result<R>
    where
        K: IntoQjAtom<'q>,
        R: FromQj<'q>,
    {
        R::from_qj(self.property(key.into_qj_atom(self.context())?)?)
    }

    #[inline]
    pub fn set_property(&self, key: Atom<'q>, val: Value<'q>) -> Result<bool> {
        Value::dup(&val);
        let ret = self.value.set_property(self.context, *key.as_raw(), *val.as_raw());
        self.context().map_err_to_exception(ret)
    }

    #[inline]
    pub fn set<K, V>(&self, key: K, val: V) -> Result<bool>
    where
        K: IntoQjAtom<'q>,
        V: IntoQj<'q>,
    {
        let ctx = self.context();
        self.set_property(key.into_qj_atom(ctx)?, val.into_qj(ctx)?)
    }

    #[inline]
    pub fn has_property(&self, key: Atom<'q>) -> Result<bool> {
        let ret = self.value.has_property(self.context, *key.as_raw());
        self.context().map_err_to_exception(ret)
    }

    #[inline]
    pub fn has_key<K>(&self, key: K) -> Result<bool>
    where
        K: IntoQjAtom<'q>,
    {
        self.has_property(key.into_qj_atom(self.context())?)
    }

    #[inline]
    pub fn is_extensible(&self) -> Result<bool> {
        self.context()
            .map_err_to_exception(self.value.is_extensible(self.context))
    }

    #[inline]
    pub fn prevent_extensions(&self) -> Result<bool> {
        self.context()
            .map_err_to_exception(self.value.prevent_extensions(self.context))
    }

    #[inline]
    pub fn own_property_names(&self, flags: GPNFlags) -> Result<Vec<PropertyEnum<'q>>> {
        if let Ok(vs) = self.value.own_property_names(self.context, flags) {
            Ok(vs
                .iter()
                .map(|v| {
                    let v = PropertyEnum::from_raw_parts(v.clone(), self.context);
                    // must be dupped
                    PropertyEnum::dup(&v);
                    v
                })
                .collect())
        } else {
            Err(Context::from_raw(self.context).internal_js_error())
        }
    }

    #[inline]
    pub fn own_property(&self, prop: Atom<'q>) -> Result<Option<PropertyDescriptor>> {
        let ret = self.value.own_property(self.context, *prop.as_raw());
        self.context()
            .map_err_to_exception(ret)
            .map(|v| v.map(|desc| PropertyDescriptor::from_raw_parts(desc, self.context)))
    }

    #[inline]
    pub fn define_property(
        &self,
        prop: Atom<'q>,
        val: Value<'q>,
        getter: Value<'q>,
        setter: Value<'q>,
        flags: PropFlags,
    ) -> Result<bool> {
        let ret = self.value.define_property(
            self.context,
            *prop.as_raw(),
            *val.as_raw(),
            *getter.as_raw(),
            *setter.as_raw(),
            flags,
        );
        self.context().map_err_to_exception(ret)
    }

    #[inline]
    pub fn define_property_value(&self, prop: Atom<'q>, val: Value<'q>, flags: PropFlags) -> Result<bool> {
        Value::dup(&val);
        let ret = self
            .value
            .define_property_value(self.context, *prop.as_raw(), *val.as_raw(), flags);
        self.context().map_err_to_exception(ret)
    }

    #[inline]
    pub fn define_property_value_with<K: IntoQjAtom<'q>, V: IntoQj<'q>>(
        &self,
        prop: K,
        val: V,
        flags: PropFlags,
    ) -> Result<bool> {
        self.define_property_value(prop.into_qj_atom(self.context())?, val.into_qj(self.context())?, flags)
    }

    #[inline]
    pub fn define_property_get_set(
        &self,
        prop: Atom<'q>,
        getter: Value<'q>,
        setter: Value<'q>,
        flags: PropFlags,
    ) -> Result<bool> {
        Value::dup(&getter);
        Value::dup(&setter);
        let ret =
            self.value
                .define_property_get_set(self.context, *prop.as_raw(), *getter.as_raw(), *setter.as_raw(), flags);
        self.context().map_err_to_exception(ret)
    }

    #[inline]
    pub fn define_property_get_set_with<K: IntoQjAtom<'q>, G: IntoQj<'q>, S: IntoQj<'q>>(
        &self,
        prop: K,
        getter: G,
        setter: S,
        flags: PropFlags,
    ) -> Result<bool> {
        self.define_property_get_set(
            prop.into_qj_atom(self.context())?,
            getter.into_qj(self.context())?,
            setter.into_qj(self.context())?,
            flags,
        )
    }

    // function

    #[inline]
    pub fn is_function(&self) -> bool {
        self.value.is_function(self.context)
    }

    #[inline]
    pub fn is_constructor(&self) -> bool {
        self.value.is_constructor(self.context)
    }

    #[inline]
    pub fn set_constructor_bit(&self, val: bool) -> Result<bool> {
        Ok(self.value.set_constructor_bit(self.context, val))
    }

    #[inline]
    pub fn set_constructor(&self, proto: Value) -> Result<()> {
        self.value.set_constructor(self.context, *proto.as_raw());
        Ok(())
    }

    // class

    #[inline]
    pub fn prototype(&self) -> Result<Value> {
        unsafe { self.context().wrap_result(self.value.prototype(self.context)) }
    }

    #[inline]
    fn opaque_internal<C: Class + 'static>(&self) -> Option<&mut C> {
        let rt = Runtime::from(self.context.runtime());
        let clz = rt.class_id::<C>()?;
        let p = self.value.opaque(clz) as *mut C;
        if p.is_null() {
            return None;
        }
        Some(unsafe { &mut *p })
    }

    #[inline]
    pub fn opaque<C: Class + 'static>(&self) -> Option<&C> {
        self.opaque_internal().map(|v| &*v)
    }

    #[inline]
    pub fn opaque_mut<C: Class + 'static>(&mut self) -> Option<&mut C> {
        self.opaque_internal()
    }

    #[inline]
    pub(crate) fn set_opaque<C: Class + 'static>(&mut self, v: C) {
        let mut rt = Runtime::from(self.context.runtime());
        let _clz = rt.get_or_register_class_id::<C>();
        let v = Box::new(v);
        // this Box will be dropped in class::finalize.
        let p = Box::into_raw(v);
        self.value.set_opaque(p as *mut c_void);
    }
}

impl Drop for Value<'_> {
    fn drop(&mut self) {
        unsafe { Self::free(self) }
    }
}

impl Clone for Value<'_> {
    fn clone(&self) -> Self {
        let qj = Value {
            value: self.value,
            context: self.context,
            #[cfg(feature = "debug_leak")]
            _debug_count: self._debug_count,
        };
        Value::dup(&qj);
        qj
    }
}

impl fmt::Debug for Value<'_> {
    #[cfg(not(feature = "debug_leak"))]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> StdResult<(), fmt::Error> {
        f.write_str(format!("Qj({:?})", self.value).as_str())
    }

    #[cfg(feature = "debug_leak")]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> StdResult<(), fmt::Error> {
        f.write_str(format!("Qj({}, {:?})", self._debug_count, self.value).as_str())
    }
}
pub struct PropertyDescriptor<'q> {
    flags: PropFlags,
    value: Value<'q>,
    getter: Value<'q>,
    setter: Value<'q>,
}

impl<'q> PropertyDescriptor<'q> {
    #[inline]
    pub(crate) fn from_raw_parts(desc: qc::PropertyDescriptor<'q>, ctx: qc::Context<'q>) -> PropertyDescriptor<'q> {
        PropertyDescriptor {
            flags: desc.flags(),
            value: Value::from_raw_parts(desc.value(), ctx),
            getter: Value::from_raw_parts(desc.getter(), ctx),
            setter: Value::from_raw_parts(desc.setter(), ctx),
        }
    }

    #[inline]
    pub fn flags(&self) -> PropFlags {
        self.flags
    }

    #[inline]
    pub fn value(&self) -> &Value<'q> {
        &self.value
    }

    #[inline]
    pub fn getter(&self) -> &Value<'q> {
        &self.getter
    }

    #[inline]
    pub fn setter(&self) -> &Value<'q> {
        &self.setter
    }
}
