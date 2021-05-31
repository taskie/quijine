use crate::{
    conversion::{AsJsCFunctionListEntry, AsJsValue},
    ffi,
    marker::Invariant,
    Context, Value,
};
use std::{ffi::CString, marker::PhantomData, os::raw::c_int, slice};

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
    let ctx = Context::from_ptr(ctx);
    let this = Value::from_raw(js_this, ctx);
    let args = slice::from_raw_parts(argv, argc as usize);
    let args: Vec<Value> = args.iter().map(|v| Value::from_raw(*v, ctx)).collect();
    (ctx, this, args)
}

/// This function is used by js_c_function macro.
#[inline]
pub fn convert_function_result(res: &Value) -> ffi::JSValue {
    res.as_js_value()
}

#[derive(Clone)]
pub struct CFunctionListEntry<'q>(ffi::JSCFunctionListEntry, CString, Invariant<'q>);

impl<'q> CFunctionListEntry<'q> {
    /// # Safety
    /// The content of JSCFunctionListEntry must have a valid lifetime.
    pub unsafe fn from_raw(raw: ffi::JSCFunctionListEntry) -> CFunctionListEntry<'q> {
        CFunctionListEntry(raw, CString::from_raw(raw.name as *mut i8), PhantomData)
    }

    /// # Safety
    /// The content of JSCFunctionListEntry must have a valid lifetime.
    pub unsafe fn from_raw_with_name(raw: ffi::JSCFunctionListEntry, name: CString) -> CFunctionListEntry<'q> {
        CFunctionListEntry(raw, name, PhantomData)
    }

    #[inline]
    pub fn new<S>(name: S, length: u8, func1: ffi::JSCFunction) -> CFunctionListEntry<'q>
    where
        S: AsRef<str>,
    {
        let c_name = CString::new(name.as_ref()).unwrap();
        let c_def = unsafe { ffi::JS_CFUNC_DEF(c_name.as_ptr(), length, func1) };
        unsafe { CFunctionListEntry::from_raw_with_name(c_def, c_name) }
    }

    #[inline]
    pub fn new_magic<S>(name: S, length: u8, func1: ffi::JSCFunctionMagic, magic: i16) -> CFunctionListEntry<'q>
    where
        S: AsRef<str>,
    {
        let c_name = CString::new(name.as_ref()).unwrap();
        let c_def = unsafe { ffi::JS_CFUNC_MAGIC_DEF(c_name.as_ptr(), length, func1, magic) };
        unsafe { CFunctionListEntry::from_raw_with_name(c_def, c_name) }
    }

    #[inline]
    pub fn new_constructor<S>(name: S, length: u8, func1: ffi::JSCFunction) -> CFunctionListEntry<'q>
    where
        S: AsRef<str>,
    {
        let c_name = CString::new(name.as_ref()).unwrap();
        let c_def = unsafe { ffi::JS_CFUNC_SPECIAL_DEF_constructor(c_name.as_ptr(), length, func1) };
        unsafe { CFunctionListEntry::from_raw_with_name(c_def, c_name) }
    }
}

impl<'q> AsJsCFunctionListEntry<'q> for CFunctionListEntry<'q> {
    #[inline]
    fn as_js_c_function_list_entry(&self) -> ffi::JSCFunctionListEntry {
        self.0
    }
}
