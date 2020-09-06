use crate::{ffi, Value};

pub trait AsJSRuntimePointer {
    fn as_ptr(&self) -> *mut ffi::JSRuntime;
}

impl AsJSRuntimePointer for *mut ffi::JSRuntime {
    fn as_ptr(&self) -> *mut ffi::JSRuntime {
        *self
    }
}

pub trait AsJSContextPointer<'q> {
    fn as_ptr(&self) -> *mut ffi::JSContext;
}

impl AsJSContextPointer<'_> for *mut ffi::JSContext {
    fn as_ptr(&self) -> *mut ffi::JSContext {
        *self
    }
}

pub trait AsJSValue<'q> {
    fn as_js_value(&self) -> ffi::JSValue;
}

impl AsJSValue<'_> for ffi::JSValue {
    fn as_js_value(&self) -> ffi::JSValue {
        *self
    }
}

pub trait AsValue<'q> {
    fn as_value(&self) -> Value<'q>;
}

impl<'q> AsValue<'q> for Value<'q> {
    fn as_value(&self) -> Value<'q> {
        *self
    }
}
