use crate::{convert::AsJsValue, ffi, raw, Context, PropFlags, Value};
use std::{ffi::CStr, os::raw::c_int, slice};

/// This function is used by js_c_function macro.
/// # Safety
/// * A context and values must have valid lifetime.
/// * The length of argv must equal to argc.
#[inline]
pub unsafe fn convert_function_arguments<'q>(
    ctx: *mut ffi::JSContext,
    js_this: ffi::JSValue,
    argc: c_int,
    argv: *mut ffi::JSValue,
) -> (Context<'q>, Value<'q>, Vec<Value<'q>>) {
    let ctx = Context::from_raw(ctx);
    let this = Value::from_raw(js_this, ctx);
    let args = slice::from_raw_parts(argv, argc as usize);
    let args: Vec<Value> = args.iter().map(|v| Value::from_raw(*v, ctx)).collect();
    (ctx, this, args)
}

/// This function is used by js_c_function macro.
#[inline]
pub fn convert_function_result(res: Value) -> ffi::JSValue {
    res.as_js_value()
}

#[derive(Clone)]
#[repr(transparent)]
pub struct CFunctionListEntry(ffi::JSCFunctionListEntry);

impl CFunctionListEntry {
    /// # Safety
    /// The content of JSCFunctionListEntry must have a valid lifetime.
    #[inline]
    pub unsafe fn from_raw(raw: ffi::JSCFunctionListEntry) -> CFunctionListEntry {
        CFunctionListEntry(raw)
    }

    /// # Safety
    /// The function pointer must be valid.
    #[deny(unsafe_op_in_unsafe_fn)]
    #[inline]
    pub unsafe fn cfunc_def(name: &CStr, length: u8, func1: ffi::JSCFunction) -> CFunctionListEntry {
        unsafe { CFunctionListEntry::from_raw(ffi::JS_CFUNC_DEF(name.as_ptr(), length, func1)) }
    }

    /// # Safety
    /// The function pointer must be valid.
    #[deny(unsafe_op_in_unsafe_fn)]
    #[inline]
    pub unsafe fn cfunc_magic_def(
        name: &CStr,
        length: u8,
        func1: ffi::JSCFunctionMagic,
        magic: i16,
    ) -> CFunctionListEntry {
        unsafe { CFunctionListEntry::from_raw(ffi::JS_CFUNC_MAGIC_DEF(name.as_ptr(), length, func1, magic)) }
    }

    /// # Safety
    /// The function pointer must be valid.
    #[deny(unsafe_op_in_unsafe_fn)]
    #[inline]
    pub unsafe fn cfunc_constructor_def(name: &CStr, length: u8, func1: ffi::JSCFunction) -> CFunctionListEntry {
        unsafe { CFunctionListEntry::from_raw(ffi::JS_CFUNC_CONSTRUCTOR_DEF(name.as_ptr(), length, func1)) }
    }

    /// # Safety
    /// The function pointer must be valid.
    #[deny(unsafe_op_in_unsafe_fn)]
    #[inline]
    pub unsafe fn cfunc_constructor_or_func_def(
        name: &CStr,
        length: u8,
        func1: ffi::JSCFunction,
    ) -> CFunctionListEntry {
        unsafe { CFunctionListEntry::from_raw(ffi::JS_CFUNC_CONSTRUCTOR_OR_FUNC_DEF(name.as_ptr(), length, func1)) }
    }

    /// # Safety
    /// The function pointer must be valid.
    #[deny(unsafe_op_in_unsafe_fn)]
    #[inline]
    pub unsafe fn cfunc_f_f_def(
        name: &CStr,
        length: u8,
        func1: Option<unsafe extern "C" fn(f64) -> f64>,
    ) -> CFunctionListEntry {
        unsafe { CFunctionListEntry::from_raw(ffi::JS_CFUNC_F_F_DEF(name.as_ptr(), length, func1)) }
    }

    /// # Safety
    /// The function pointer must be valid.
    #[deny(unsafe_op_in_unsafe_fn)]
    #[inline]
    pub unsafe fn cfunc_f_f_f_def(
        name: &CStr,
        length: u8,
        func1: Option<unsafe extern "C" fn(f64, f64) -> f64>,
    ) -> CFunctionListEntry {
        unsafe { CFunctionListEntry::from_raw(ffi::JS_CFUNC_F_F_F_DEF(name.as_ptr(), length, func1)) }
    }

    /// # Safety
    /// The function pointer must be valid.
    #[deny(unsafe_op_in_unsafe_fn)]
    #[inline]
    pub unsafe fn iterator_next_def(
        name: &CStr,
        length: u8,
        func1: Option<
            unsafe extern "C" fn(
                *mut raw::JSContext,
                raw::JSValue,
                c_int,
                *mut raw::JSValue,
                *mut c_int,
                c_int,
            ) -> raw::JSValue,
        >,
        magic: i16,
    ) -> CFunctionListEntry {
        unsafe { CFunctionListEntry::from_raw(ffi::JS_ITERATOR_NEXT_DEF(name.as_ptr(), length, func1, magic)) }
    }

    /// # Safety
    /// The function pointer must be valid.
    #[deny(unsafe_op_in_unsafe_fn)]
    #[inline]
    pub unsafe fn cgetset_def(
        name: &CStr,
        fgetter: Option<unsafe extern "C" fn(*mut raw::JSContext, raw::JSValue) -> raw::JSValue>,
        fsetter: Option<unsafe extern "C" fn(*mut raw::JSContext, raw::JSValue, raw::JSValue) -> raw::JSValue>,
    ) -> CFunctionListEntry {
        unsafe { CFunctionListEntry::from_raw(ffi::JS_CGETSET_DEF(name.as_ptr(), fgetter, fsetter)) }
    }

    /// # Safety
    /// The function pointer must be valid.
    #[deny(unsafe_op_in_unsafe_fn)]
    #[inline]
    pub unsafe fn cgetset_magic_def(
        name: &CStr,
        fgetter: Option<unsafe extern "C" fn(*mut raw::JSContext, raw::JSValue, c_int) -> raw::JSValue>,
        fsetter: Option<unsafe extern "C" fn(*mut raw::JSContext, raw::JSValue, raw::JSValue, c_int) -> raw::JSValue>,
        magic: i16,
    ) -> CFunctionListEntry {
        unsafe { CFunctionListEntry::from_raw(ffi::JS_CGETSET_MAGIC_DEF(name.as_ptr(), fgetter, fsetter, magic)) }
    }

    #[inline]
    pub fn prop_string_def(name: &CStr, cstr: &CStr, prop_flags: PropFlags) -> CFunctionListEntry {
        unsafe {
            CFunctionListEntry::from_raw(ffi::JS_PROP_STRING_DEF(name.as_ptr(), cstr.as_ptr(), prop_flags.bits()))
        }
    }

    #[inline]
    pub fn prop_int32_def(name: &CStr, val: i32, prop_flags: PropFlags) -> CFunctionListEntry {
        unsafe { CFunctionListEntry::from_raw(ffi::JS_PROP_INT32_DEF(name.as_ptr(), val, prop_flags.bits())) }
    }

    #[inline]
    pub fn prop_int64_def(name: &CStr, val: i64, prop_flags: PropFlags) -> CFunctionListEntry {
        unsafe { CFunctionListEntry::from_raw(ffi::JS_PROP_INT64_DEF(name.as_ptr(), val, prop_flags.bits())) }
    }

    #[inline]
    pub fn prop_double_def(name: &CStr, val: f64, prop_flags: PropFlags) -> CFunctionListEntry {
        unsafe { CFunctionListEntry::from_raw(ffi::JS_PROP_DOUBLE_DEF(name.as_ptr(), val, prop_flags.bits())) }
    }

    #[inline]
    pub fn prop_undefined_def(name: &CStr, prop_flags: PropFlags) -> CFunctionListEntry {
        unsafe { CFunctionListEntry::from_raw(ffi::JS_PROP_UNDEFINED_DEF(name.as_ptr(), prop_flags.bits())) }
    }

    #[inline]
    pub fn object_def(name: &CStr, tab: &[CFunctionListEntry], prop_flags: PropFlags) -> CFunctionListEntry {
        unsafe {
            CFunctionListEntry::from_raw(ffi::JS_OBJECT_DEF(
                name.as_ptr(),
                c_function_list_as_ptr(tab),
                tab.len() as i32,
                prop_flags.bits(),
            ))
        }
    }

    #[inline]
    pub fn alias_def(name: &CStr, from: &CStr) -> CFunctionListEntry {
        unsafe { CFunctionListEntry::from_raw(ffi::JS_ALIAS_DEF(name.as_ptr(), from.as_ptr())) }
    }

    #[inline]
    pub fn alias_base_def(name: &CStr, from: &CStr, base: i32) -> CFunctionListEntry {
        unsafe { CFunctionListEntry::from_raw(ffi::JS_ALIAS_BASE_DEF(name.as_ptr(), from.as_ptr(), base)) }
    }
}

pub(crate) fn c_function_list_as_ptr(list: &[CFunctionListEntry]) -> *const ffi::JSCFunctionListEntry {
    // this operation is safe because of repr(transparent)
    list.as_ptr() as *const _
}

impl AsRef<ffi::JSCFunctionListEntry> for CFunctionListEntry {
    #[inline]
    fn as_ref(&self) -> &ffi::JSCFunctionListEntry {
        &self.0
    }
}
