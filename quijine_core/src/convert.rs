use std::os::raw::c_char;

use crate::{ffi, Value};

pub trait AsJsRuntimePointer {
    fn as_ptr(&self) -> *mut ffi::JSRuntime;
}

impl AsJsRuntimePointer for *mut ffi::JSRuntime {
    #[inline]
    fn as_ptr(&self) -> *mut ffi::JSRuntime {
        *self
    }
}

pub trait AsJsContextPointer<'q> {
    fn as_ptr(&self) -> *mut ffi::JSContext;
}

impl AsJsContextPointer<'_> for *mut ffi::JSContext {
    #[inline]
    fn as_ptr(&self) -> *mut ffi::JSContext {
        *self
    }
}

pub trait AsJsValue<'q> {
    fn as_js_value(&self) -> ffi::JSValue;
}

impl AsJsValue<'_> for ffi::JSValue {
    #[inline]
    fn as_js_value(&self) -> ffi::JSValue {
        *self
    }
}

pub trait AsJsClassId<'q> {
    fn as_js_class_id(&self) -> ffi::JSClassID;
}

impl AsJsClassId<'_> for ffi::JSClassID {
    #[inline]
    fn as_js_class_id(&self) -> ffi::JSClassID {
        *self
    }
}

pub trait AsJsAtom<'q> {
    fn as_js_atom(&self) -> ffi::JSAtom;
}

impl AsJsAtom<'_> for ffi::JSAtom {
    #[inline]
    fn as_js_atom(&self) -> ffi::JSAtom {
        *self
    }
}

pub trait AsJsCFunctionListEntry<'q> {
    fn as_js_c_function_list_entry(&self) -> ffi::JSCFunctionListEntry;
}

impl AsJsCFunctionListEntry<'_> for ffi::JSCFunctionListEntry {
    #[inline]
    fn as_js_c_function_list_entry(&self) -> ffi::JSCFunctionListEntry {
        *self
    }
}

pub trait AsValue<'q> {
    fn as_ref(&self) -> Value<'q>;
}

impl<'q> AsValue<'q> for Value<'q> {
    #[inline]
    fn as_ref(&self) -> Value<'q> {
        *self
    }
}

pub trait AsJsCString<'q> {
    fn as_js_c_string(&self) -> *const c_char;
}

impl AsJsCString<'_> for *const c_char {
    #[inline]
    fn as_js_c_string(&self) -> *const c_char {
        *self
    }
}
