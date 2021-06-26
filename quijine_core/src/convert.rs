use crate::ffi;

pub trait AsPtr<T> {
    fn as_ptr(&self) -> *const T;
}

impl<T> AsPtr<T> for *const T {
    #[inline]
    fn as_ptr(&self) -> *const T {
        *self
    }
}

impl<T> AsPtr<T> for *mut T {
    #[inline]
    fn as_ptr(&self) -> *const T {
        *self
    }
}

pub trait AsMutPtr<T>: AsPtr<T> {
    fn as_mut_ptr(&mut self) -> *mut T;
}

impl<T> AsMutPtr<T> for *mut T {
    #[inline]
    fn as_mut_ptr(&mut self) -> *mut T {
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
