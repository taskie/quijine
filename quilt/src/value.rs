use crate::{
    atom::Atom,
    class::ClassId,
    context::Context,
    conversion::{AsJsCFunctionListEntry, AsJsClassId, AsJsContextPointer, AsJsValue},
    ffi,
    function::CFunctionListEntry,
    marker::Covariant,
    string::CString as LtCString,
    util, Runtime,
};
use log::trace;
use std::{
    ffi::{c_void, CString},
    fmt,
    marker::PhantomData,
    mem::size_of,
    os::raw::c_int,
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

    #[inline]
    pub unsafe fn from_raw(value: ffi::JSValue, _ctx: Context<'q>) -> Value<'q> {
        Value(value, PhantomData)
    }

    #[inline]
    pub unsafe fn from_raw_with_runtime(value: ffi::JSValue, _rt: Runtime<'q>) -> Value<'q> {
        Value(value, PhantomData)
    }

    #[inline]
    pub unsafe fn from_static_raw(value: ffi::JSValue) -> Value<'static> {
        Value(value, PhantomData)
    }

    pub fn has_ref_count(self) -> bool {
        unsafe { ffi::JS_VALUE_HAS_REF_COUNT(self.0) }
    }

    pub fn ref_count(self) -> Option<usize> {
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
    pub fn tag(self) -> i32 {
        unsafe { ffi::JS_VALUE_GET_TAG(self.0) }
    }

    // special values

    #[inline]
    pub fn undefined() -> Value<'static> {
        unsafe { Value::from_static_raw(ffi::JS_UNDEFINED) }
    }

    #[inline]
    pub fn null() -> Value<'static> {
        unsafe { Value::from_static_raw(ffi::JS_NULL) }
    }

    #[inline]
    pub fn exception() -> Value<'static> {
        unsafe { Value::from_static_raw(ffi::JS_EXCEPTION) }
    }

    // conversion

    #[inline]
    pub fn to_bool(self, ctx: Context<'q>) -> Option<bool> {
        let v = unsafe { ffi::JS_ToBool(ctx.as_ptr(), self.0) };
        if v == -1 {
            None
        } else if v == 0 {
            Some(false)
        } else {
            Some(true)
        }
    }

    #[inline]
    pub fn to_i32(self, ctx: Context<'q>) -> Option<i32> {
        let mut v = 0;
        let x = unsafe { ffi::JS_ToInt32(ctx.as_ptr(), &mut v as *mut i32, self.0) };
        if x == 0 {
            Some(v)
        } else {
            None
        }
    }

    #[inline]
    pub fn to_u32(self, ctx: Context<'q>) -> Option<u32> {
        let mut v = 0u32;
        let x = unsafe { ffi::JS_ToUint32(ctx.as_ptr(), &mut v as *mut u32, self.0) };
        if x == 0 {
            Some(v)
        } else {
            None
        }
    }

    #[inline]
    pub fn to_i64(self, ctx: Context<'q>) -> Option<i64> {
        let mut v = 0;
        let x = unsafe { ffi::JS_ToInt64(ctx.as_ptr(), &mut v as *mut i64, self.0) };
        if x == 0 {
            Some(v)
        } else {
            None
        }
    }

    #[inline]
    pub fn to_f64(self, ctx: Context<'q>) -> Option<f64> {
        let mut v = 0.0;
        let x = unsafe { ffi::JS_ToFloat64(ctx.as_ptr(), &mut v as *mut f64, self.0) };
        if x == 0 {
            Some(v)
        } else {
            None
        }
    }

    #[inline]
    pub fn to_c_string(self, ctx: Context<'q>) -> Option<LtCString<'q>> {
        let c_str = unsafe { ffi::JS_ToCString(ctx.as_ptr(), self.0) };
        if !c_str.is_null() {
            Some(LtCString::new(c_str))
        } else {
            None
        }
    }

    // is * (object)

    #[inline]
    pub fn is_function(self, ctx: Context<'q>) -> bool {
        unsafe { ffi::JS_IsFunction(ctx.as_ptr(), self.0) != 0 }
    }

    #[inline]
    pub fn is_array(self, ctx: Context<'q>) -> bool {
        unsafe { ffi::JS_IsArray(ctx.as_ptr(), self.0) != 0 }
    }

    // property

    #[inline]
    pub fn property(self, ctx: Context<'q>, prop: Atom<'q>) -> Value<'q> {
        unsafe {
            let value = ffi::JS_GetProperty(ctx.as_ptr(), self.0, Atom::raw(prop));
            Value::from_raw(value, ctx)
        }
    }

    #[inline]
    pub fn property_str<K>(self, ctx: Context<'q>, key: K) -> Value<'q>
    where
        K: AsRef<str>,
    {
        let c_key = CString::new(key.as_ref()).unwrap();
        unsafe {
            let value = ffi::JS_GetPropertyStr(ctx.as_ptr(), self.0, c_key.as_ptr());
            Value::from_raw(value, ctx)
        }
    }

    #[inline]
    pub fn set_property<K, V>(self, ctx: Context<'q>, prop: Atom<'q>, val: V)
    where
        V: AsJsValue<'q>,
    {
        unsafe { ffi::JS_SetProperty(ctx.as_ptr(), self.0, Atom::raw(prop), val.as_js_value()) };
    }

    #[inline]
    pub fn set_property_str<K, V>(self, ctx: Context<'q>, key: K, val: V)
    where
        K: AsRef<str>,
        V: AsJsValue<'q>,
    {
        let c_key = CString::new(key.as_ref()).unwrap();
        unsafe { ffi::JS_SetPropertyStr(ctx.as_ptr(), self.0, c_key.as_ptr(), val.as_js_value()) };
    }

    pub fn set_prototype<V>(self, ctx: Context<'q>, proto_val: V)
    where
        V: AsJsValue<'q>,
    {
        unsafe {
            ffi::JS_SetPrototype(ctx.as_ptr(), self.0, proto_val.as_js_value());
        }
    }

    pub fn prototype(self, ctx: Context<'q>) -> Value<'q> {
        unsafe {
            let val = ffi::JS_GetPrototype(ctx.as_ptr(), self.0);
            Value::from_raw(val, ctx)
        }
    }

    // class

    #[inline]
    pub fn opaque(self, clz: ClassId) -> *mut c_void {
        unsafe { ffi::JS_GetOpaque(self.0, clz.as_js_class_id()) }
    }

    #[inline]
    pub fn set_opaque(self, opaque: *mut c_void) {
        unsafe { ffi::JS_SetOpaque(self.0, opaque) }
    }

    // array buffer

    pub unsafe fn array_buffer(self, ctx: Context<'q>) -> Option<&[u8]> {
        let mut len = 0;
        let bs: *const u8 = ffi::JS_GetArrayBuffer(ctx.as_ptr(), &mut len, self.0);
        if bs.is_null() {
            return None;
        }
        Some(slice::from_raw_parts(bs, len as usize))
    }

    pub unsafe fn array_buffer_to_sized<T>(self, ctx: Context<'q>) -> Option<&T> {
        let mut len = 0;
        let bs: *const u8 = ffi::JS_GetArrayBuffer(ctx.as_ptr(), &mut len, self.0);
        if bs.is_null() {
            return None;
        }
        assert_eq!(size_of::<T>(), len as usize);
        Some(&*(bs as *const T))
    }

    // C property

    #[inline]
    pub fn set_property_function_list(self, ctx: Context, tab: &[CFunctionListEntry<'q>]) {
        let tab: Vec<ffi::JSCFunctionListEntry> = tab.iter().map(|v| v.as_js_c_function_list_entry()).collect();
        trace!("length: {}", tab.len());
        unsafe { ffi::JS_SetPropertyFunctionList(ctx.as_ptr(), self.0, tab.as_ptr(), tab.len() as c_int) }
    }
}

impl fmt::Debug for Value<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let mut repr = String::new();
        let tag = self.tag();
        for x in util::to_vec(self.0) {
            repr.push_str(format!("{:02x}", x).as_str())
        }
        f.write_str(format!("Value(tag={}, {})", tag, repr).as_str())
    }
}

impl<'q> AsJsValue<'q> for Value<'q> {
    #[inline]
    fn as_js_value(&self) -> ffi::JSValue {
        self.0
    }
}
