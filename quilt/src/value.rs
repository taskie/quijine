use crate::{
    class::ClassId,
    context::{AsJSContextPointer, Context},
    conversion::AsJSValue,
    ffi,
    marker::Covariant,
    string::CString as CoreCString,
    util, Runtime,
};
use std::{
    ffi::{c_void, CString},
    fmt,
    marker::PhantomData,
};

#[derive(Clone, Copy)]
pub struct Value<'q>(ffi::JSValue, Covariant<'q>);

impl<'q> Value<'q> {
    #[inline]
    pub fn from_raw(value: ffi::JSValue, _ctx: Context<'q>) -> Value<'q> {
        Value(value, PhantomData)
    }

    #[inline]
    pub fn from_raw_with_runtime(value: ffi::JSValue, _ctx: Runtime<'q>) -> Value<'q> {
        Value(value, PhantomData)
    }

    #[inline]
    pub fn from_static_raw(value: ffi::JSValue) -> Value<'static> {
        Value(value, PhantomData)
    }

    #[inline]
    pub fn raw(this: Self) -> ffi::JSValue {
        this.0
    }

    // memory

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

    #[inline]
    pub fn is_null(self) -> bool {
        unsafe { ffi::JS_IsNull(self.0) }
    }

    #[inline]
    pub fn is_exception(self) -> bool {
        unsafe { ffi::JS_IsException(self.0) }
    }

    #[inline]
    pub fn is_undefined(self) -> bool {
        unsafe { ffi::JS_IsUndefined(self.0) }
    }

    #[inline]
    pub fn is_uninitialized(self) -> bool {
        unsafe { ffi::JS_IsUninitialized(self.0) }
    }

    // special values

    #[inline]
    pub fn undefined() -> Value<'static> {
        Value::from_static_raw(ffi::JS_UNDEFINED)
    }

    #[inline]
    pub fn null() -> Value<'static> {
        Value::from_static_raw(ffi::JS_NULL)
    }

    #[inline]
    pub fn exception() -> Value<'static> {
        Value::from_static_raw(ffi::JS_EXCEPTION)
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

    pub fn to_c_string(self, ctx: Context<'q>) -> Option<CoreCString<'q>> {
        let c_str = unsafe { ffi::JS_ToCString(ctx.as_ptr(), self.0) };
        if !c_str.is_null() {
            Some(CoreCString::new(c_str))
        } else {
            None
        }
    }

    // property

    #[inline]
    pub fn property<K>(self, ctx: Context<'q>, key: K) -> Value<'q>
    where
        K: AsRef<str>,
    {
        let c_key = CString::new(key.as_ref()).unwrap();
        let value = unsafe { ffi::JS_GetPropertyStr(ctx.as_ptr(), self.as_js_value(), c_key.as_ptr()) };
        Value::from_raw(value, ctx)
    }

    #[inline]
    pub fn set_property<K, V>(self, ctx: Context<'q>, key: K, val: V)
    where
        K: AsRef<str>,
        V: AsJSValue<'q>,
    {
        let c_key = CString::new(key.as_ref()).unwrap();
        unsafe { ffi::JS_SetPropertyStr(ctx.as_ptr(), self.0, c_key.as_ptr(), val.as_js_value()) };
    }

    // class

    #[inline]
    pub fn opaque(self, clz: ClassId) -> *mut c_void {
        unsafe { ffi::JS_GetOpaque(self.0, ClassId::raw(clz)) }
    }

    #[inline]
    pub fn set_opaque(self, opaque: *mut c_void) {
        unsafe { ffi::JS_SetOpaque(self.0, opaque) }
    }
}

impl fmt::Debug for Value<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let mut repr = String::new();
        let tag = self.tag();
        for x in util::to_vec(self.0) {
            repr.push_str(format!("{:02x}", x).as_str())
        }
        f.write_str(format!("QjValueTag(tag={}, {})", tag, repr).as_str())
    }
}
