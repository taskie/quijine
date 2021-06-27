use crate::{
    atom::{Atom, PropertyEnum},
    class::ClassId,
    context::Context,
    convert::{AsJsAtom, AsJsClassId, AsJsValue, AsMutPtr},
    enums::ValueTag,
    error::Error,
    ffi,
    flags::{GPNFlags, PropFlags},
    internal::{ref_sized_from_bytes, ref_sized_to_vec},
    marker::Covariant,
    runtime::Runtime,
    string::CString as QcCString,
};
use log::trace;
use std::{
    ffi::{c_void, CString},
    fmt,
    marker::PhantomData,
    mem::{transmute, MaybeUninit},
    os::raw::c_int,
    ptr::null_mut,
    slice,
};

#[derive(Clone, Copy)]
pub struct Value<'q>(ffi::JSValue, Covariant<'q>);

macro_rules! def_is_some {
    ($name: ident, $f: expr) => {
        #[inline]
        pub fn $name(self) -> bool {
            unsafe { $f(self.0) != 0 }
        }
    };
}

impl<'q> Value<'q> {
    // is *
    def_is_some!(is_number, ffi::JS_IsNumber);

    def_is_some!(is_big_int, ffi::JS_IsBigInt);

    def_is_some!(is_big_float, ffi::JS_IsBigFloat);

    def_is_some!(is_big_decimal, ffi::JS_IsBigDecimal);

    def_is_some!(is_bool, ffi::JS_IsBool);

    def_is_some!(is_null, ffi::JS_IsNull);

    def_is_some!(is_undefined, ffi::JS_IsUndefined);

    def_is_some!(is_exception, ffi::JS_IsException);

    def_is_some!(is_uninitialized, ffi::JS_IsUninitialized);

    def_is_some!(is_string, ffi::JS_IsString);

    def_is_some!(is_symbol, ffi::JS_IsSymbol);

    def_is_some!(is_object, ffi::JS_IsObject);

    // lifecycle

    /// # Safety
    /// value must have the same lifetime as a context.
    #[inline]
    pub unsafe fn from_raw(value: ffi::JSValue, _ctx: Context<'q>) -> Value<'q> {
        Value(value, PhantomData)
    }

    /// # Safety
    /// value must have the same lifetime as a runtime.
    #[inline]
    pub unsafe fn from_raw_with_runtime(value: ffi::JSValue, _rt: Runtime<'q>) -> Value<'q> {
        Value(value, PhantomData)
    }

    /// # Safety
    /// value must be the "value" type, not the reference type.
    #[inline]
    pub unsafe fn from_raw_static(value: ffi::JSValue) -> Value<'static> {
        Value(value, PhantomData)
    }

    #[inline]
    pub fn has_ref_count(self) -> bool {
        unsafe { ffi::JS_VALUE_HAS_REF_COUNT(self.0) }
    }

    /// API for debug.
    pub fn debug_ref_count(self) -> Option<usize> {
        if !self.has_ref_count() {
            return None;
        }
        unsafe {
            let p = ffi::JS_VALUE_GET_PTR(self.0) as *mut ffi::JSRefCountHeader;
            let pref: &mut ffi::JSRefCountHeader = &mut *p;
            Some(pref.ref_count as usize)
        }
    }

    // type

    #[inline]
    pub fn tag(self) -> ValueTag {
        unsafe { transmute(ffi::JS_VALUE_GET_TAG(self.0)) }
    }

    #[inline]
    pub fn ptr(self) -> Option<*mut c_void> {
        if !self.has_ref_count() {
            return None;
        }
        Some(unsafe { ffi::JS_VALUE_GET_PTR(self.0) })
    }

    // special values

    #[inline]
    pub fn undefined() -> Value<'static> {
        unsafe { Value::from_raw_static(ffi::JS_UNDEFINED) }
    }

    #[inline]
    pub fn null() -> Value<'static> {
        unsafe { Value::from_raw_static(ffi::JS_NULL) }
    }

    #[inline]
    pub fn exception() -> Value<'static> {
        unsafe { Value::from_raw_static(ffi::JS_EXCEPTION) }
    }

    // conversion

    #[inline]
    pub fn to_bool(self, mut ctx: Context<'q>) -> Option<bool> {
        let v = unsafe { ffi::JS_ToBool(ctx.as_mut_ptr(), self.0) };
        if v == -1 {
            None
        } else if v == 0 {
            Some(false)
        } else {
            Some(true)
        }
    }

    #[inline]
    pub fn to_i32(self, mut ctx: Context<'q>) -> Option<i32> {
        let mut v = 0;
        let x = unsafe { ffi::JS_ToInt32(ctx.as_mut_ptr(), &mut v as *mut i32, self.0) };
        if x == 0 {
            Some(v)
        } else {
            None
        }
    }

    #[inline]
    pub fn to_u32(self, mut ctx: Context<'q>) -> Option<u32> {
        let mut v = 0u32;
        let x = unsafe { ffi::JS_ToUint32(ctx.as_mut_ptr(), &mut v as *mut u32, self.0) };
        if x == 0 {
            Some(v)
        } else {
            None
        }
    }

    #[inline]
    pub fn to_i64(self, mut ctx: Context<'q>) -> Option<i64> {
        let mut v = 0;
        let x = unsafe { ffi::JS_ToInt64(ctx.as_mut_ptr(), &mut v as *mut i64, self.0) };
        if x == 0 {
            Some(v)
        } else {
            None
        }
    }

    #[inline]
    pub fn to_f64(self, mut ctx: Context<'q>) -> Option<f64> {
        let mut v = 0.0;
        let x = unsafe { ffi::JS_ToFloat64(ctx.as_mut_ptr(), &mut v as *mut f64, self.0) };
        if x == 0 {
            Some(v)
        } else {
            None
        }
    }

    #[inline]
    pub fn to_c_string(self, mut ctx: Context<'q>) -> Option<QcCString<'q>> {
        QcCString::new(unsafe { ffi::JS_ToCString(ctx.as_mut_ptr(), self.0) })
    }

    // function

    #[inline]
    pub fn is_function(self, mut ctx: Context<'q>) -> bool {
        unsafe { ffi::JS_IsFunction(ctx.as_mut_ptr(), self.0) != 0 }
    }

    #[inline]
    pub fn is_constructor(self, mut ctx: Context<'q>) -> bool {
        unsafe { ffi::JS_IsConstructor(ctx.as_mut_ptr(), self.0) != 0 }
    }

    #[inline]
    pub fn set_constructor(self, mut ctx: Context<'q>, proto: Value) {
        unsafe { ffi::JS_SetConstructor(ctx.as_mut_ptr(), self.0, proto.0) }
    }

    #[inline]
    pub fn set_constructor_bit(self, mut ctx: Context<'q>, val: bool) -> bool {
        unsafe { ffi::JS_SetConstructorBit(ctx.as_mut_ptr(), self.0, val as c_int) != 0 }
    }

    // array

    #[inline]
    pub fn is_array(self, mut ctx: Context<'q>) -> bool {
        unsafe { ffi::JS_IsArray(ctx.as_mut_ptr(), self.0) != 0 }
    }

    // error

    #[inline]
    pub fn is_error(self, mut ctx: Context<'q>) -> bool {
        unsafe { ffi::JS_IsError(ctx.as_mut_ptr(), self.0) != 0 }
    }

    // property

    #[inline]
    pub fn property(self, mut ctx: Context<'q>, prop: Atom<'q>) -> Value<'q> {
        unsafe {
            let value = ffi::JS_GetProperty(ctx.as_mut_ptr(), self.0, prop.as_js_atom());
            Value::from_raw(value, ctx)
        }
    }

    #[inline]
    pub fn property_str(self, mut ctx: Context<'q>, key: &str) -> Value<'q> {
        let c_key = CString::new(key).unwrap();
        unsafe {
            let value = ffi::JS_GetPropertyStr(ctx.as_mut_ptr(), self.0, c_key.as_ptr());
            Value::from_raw(value, ctx)
        }
    }

    #[inline]
    pub fn set_property<V>(self, mut ctx: Context<'q>, prop: Atom<'q>, val: V) -> Result<bool, Error>
    where
        V: AsJsValue<'q>,
    {
        let ret = unsafe { ffi::JS_SetProperty(ctx.as_mut_ptr(), self.0, prop.as_js_atom(), val.as_js_value()) };
        if ret == -1 {
            Err(Error::HasException)
        } else {
            Ok(ret != 0)
        }
    }

    #[inline]
    pub fn has_property(self, mut ctx: Context<'q>, prop: Atom<'q>) -> Result<bool, Error> {
        let ret = unsafe { ffi::JS_HasProperty(ctx.as_mut_ptr(), self.0, prop.as_js_atom()) };
        if ret == -1 {
            Err(Error::HasException)
        } else {
            Ok(ret != 0)
        }
    }

    #[inline]
    pub fn set_property_str<V>(self, mut ctx: Context<'q>, key: &str, val: V) -> Result<bool, Error>
    where
        V: AsJsValue<'q>,
    {
        let c_key = CString::new(key).unwrap();
        let ret = unsafe { ffi::JS_SetPropertyStr(ctx.as_mut_ptr(), self.0, c_key.as_ptr(), val.as_js_value()) };
        if ret == -1 {
            Err(Error::HasException)
        } else {
            Ok(ret != 0)
        }
    }

    /// return `None` if exception (Proxy object only)
    #[inline]
    pub fn is_extensible(self, mut ctx: Context<'q>) -> Result<bool, Error> {
        let ret = unsafe { ffi::JS_IsExtensible(ctx.as_mut_ptr(), self.0) };
        if ret == -1 {
            Err(Error::HasException)
        } else {
            Ok(ret != 0)
        }
    }

    /// return `None` if exception (Proxy object only)
    #[inline]
    pub fn prevent_extensions(self, mut ctx: Context<'q>) -> Result<bool, Error> {
        let ret = unsafe { ffi::JS_PreventExtensions(ctx.as_mut_ptr(), self.0) };
        if ret == -1 {
            Err(Error::HasException)
        } else {
            Ok(ret != 0)
        }
    }

    #[inline]
    pub fn own_property_names(self, mut ctx: Context<'q>, flags: GPNFlags) -> Result<Vec<PropertyEnum<'q>>, Error> {
        let mut ptab: *mut ffi::JSPropertyEnum = null_mut();
        let mut plen: u32 = 0;
        let ret = unsafe {
            ffi::JS_GetOwnPropertyNames(
                ctx.as_mut_ptr(),
                &mut ptab,
                &mut plen,
                self.as_js_value(),
                flags.bits() as c_int,
            )
        };
        if ret == -1 {
            return Err(Error::HasException);
        }
        let enums = unsafe { std::slice::from_raw_parts(ptab, plen as usize) };
        let ret = enums
            .iter()
            .map(|v| unsafe { PropertyEnum::from_raw(*v, ctx) })
            .collect();
        // see js_free_prop_enum
        unsafe {
            ffi::js_free(ctx.as_mut_ptr(), ptab as *mut c_void);
        }
        Ok(ret)
    }

    #[inline]
    pub fn own_property(self, mut ctx: Context<'q>, prop: Atom<'q>) -> Result<Option<PropertyDescriptor<'q>>, Error> {
        let mut desc = MaybeUninit::<ffi::JSPropertyDescriptor>::zeroed();
        let ret = unsafe { ffi::JS_GetOwnProperty(ctx.as_mut_ptr(), desc.as_mut_ptr(), self.0, prop.as_js_atom()) };
        if ret == -1 {
            Err(Error::HasException)
        } else if ret == 0 {
            Ok(None)
        } else {
            Ok(Some(PropertyDescriptor::from_raw(unsafe { desc.assume_init() }, ctx)))
        }
    }

    #[inline]
    pub fn define_property(
        self,
        mut ctx: Context<'q>,
        prop: Atom<'q>,
        val: Value<'q>,
        getter: Value<'q>,
        setter: Value<'q>,
        flags: PropFlags,
    ) -> Result<bool, Error> {
        let ret = unsafe {
            ffi::JS_DefineProperty(
                ctx.as_mut_ptr(),
                self.0,
                prop.as_js_atom(),
                val.0,
                getter.0,
                setter.0,
                flags.bits() as c_int,
            )
        };
        if ret == -1 {
            Err(Error::HasException)
        } else {
            Ok(ret != 0)
        }
    }

    /// shortcut to add or redefine a new property value
    #[inline]
    pub fn define_property_value(
        self,
        mut ctx: Context<'q>,
        prop: Atom<'q>,
        val: Value<'q>,
        flags: PropFlags,
    ) -> Result<bool, Error> {
        let ret = unsafe {
            ffi::JS_DefinePropertyValue(
                ctx.as_mut_ptr(),
                self.0,
                prop.as_js_atom(),
                val.0,
                flags.bits() as c_int,
            )
        };
        if ret == -1 {
            Err(Error::HasException)
        } else {
            Ok(ret != 0)
        }
    }

    /// shortcut to add getter & setter
    #[inline]
    pub fn define_property_get_set(
        self,
        mut ctx: Context<'q>,
        prop: Atom<'q>,
        getter: Value<'q>,
        setter: Value<'q>,
        flags: PropFlags,
    ) -> Result<bool, Error> {
        let ret = unsafe {
            ffi::JS_DefinePropertyGetSet(
                ctx.as_mut_ptr(),
                self.0,
                prop.as_js_atom(),
                getter.0,
                setter.0,
                flags.bits() as c_int,
            )
        };
        if ret == -1 {
            Err(Error::HasException)
        } else {
            Ok(ret != 0)
        }
    }

    #[inline]
    pub fn set_prototype<V>(self, mut ctx: Context<'q>, proto_val: V) -> Result<bool, Error>
    where
        V: AsJsValue<'q>,
    {
        let ret = unsafe { ffi::JS_SetPrototype(ctx.as_mut_ptr(), self.0, proto_val.as_js_value()) };
        if ret == -1 {
            Err(Error::HasException)
        } else {
            Ok(ret != 0)
        }
    }

    #[inline]
    pub fn prototype(self, mut ctx: Context<'q>) -> Value<'q> {
        unsafe {
            let val = ffi::JS_GetPrototype(ctx.as_mut_ptr(), self.0);
            Value::from_raw(val, ctx)
        }
    }

    // class

    #[inline]
    pub fn opaque(self, clz: ClassId) -> *mut c_void {
        unsafe { ffi::JS_GetOpaque(self.0, clz.as_js_class_id()) }
    }

    // QuickJS C library doesn't dereference an opaque.
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    #[inline]
    pub fn set_opaque(self, opaque: *mut c_void) {
        unsafe { ffi::JS_SetOpaque(self.0, opaque) }
    }

    // array buffer

    #[inline]
    pub fn array_buffer(self, mut ctx: Context<'q>) -> Option<&[u8]> {
        let mut len = 0;
        let bs: *const u8 = unsafe { ffi::JS_GetArrayBuffer(ctx.as_mut_ptr(), &mut len, self.0) };
        if bs.is_null() {
            return None;
        }
        Some(unsafe { slice::from_raw_parts(bs, len as usize) })
    }

    /// # Safety
    /// The content of ArrayBuffer must be created from `T`.
    #[inline]
    pub unsafe fn array_buffer_as_ref<T>(self, ctx: Context<'q>) -> Option<&T> {
        self.array_buffer(ctx).map(|v| ref_sized_from_bytes(v))
    }

    // C property

    #[inline]
    pub fn set_property_function_list(self, mut ctx: Context, tab: &[ffi::JSCFunctionListEntry]) {
        trace!("length: {}", tab.len());
        unsafe { ffi::JS_SetPropertyFunctionList(ctx.as_mut_ptr(), self.0, tab.as_ptr(), tab.len() as c_int) }
    }
}

impl fmt::Debug for Value<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let mut repr = String::new();
        let tag = self.tag();
        for x in ref_sized_to_vec(&self.0) {
            repr.push_str(format!("{:02x}", x).as_str())
        }
        f.write_str(format!("Value(tag={:?}, {})", tag, repr).as_str())
    }
}

impl<'q> AsJsValue<'q> for Value<'q> {
    #[inline]
    fn as_js_value(&self) -> ffi::JSValue {
        self.0
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
    pub(crate) fn from_raw(desc: ffi::JSPropertyDescriptor, ctx: Context<'q>) -> PropertyDescriptor<'q> {
        PropertyDescriptor {
            flags: PropFlags::from_bits(desc.flags as u32).unwrap(),
            value: unsafe { Value::from_raw(desc.value, ctx) },
            getter: unsafe { Value::from_raw(desc.getter, ctx) },
            setter: unsafe { Value::from_raw(desc.setter, ctx) },
        }
    }

    #[inline]
    pub fn flags(&self) -> PropFlags {
        self.flags
    }

    #[inline]
    pub fn value(&self) -> Value<'q> {
        self.value
    }

    #[inline]
    pub fn getter(&self) -> Value<'q> {
        self.getter
    }

    #[inline]
    pub fn setter(&self) -> Value<'q> {
        self.setter
    }
}
