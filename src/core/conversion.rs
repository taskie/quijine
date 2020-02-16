use crate::core::{ffi, Value};

pub trait AsValue<'q> {
    fn as_value(&self) -> Value<'q>;
}

impl<'q> AsValue<'q> for Value<'q> {
    fn as_value(&self) -> Value<'q> {
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

impl<'q> AsJSValue<'q> for Value<'q> {
    fn as_js_value(&self) -> ffi::JSValue {
        Value::raw(*self)
    }
}
