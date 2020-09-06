use crate::{
    class::{ClassDef, ClassId},
    conversion::{AsJSRuntimePointer, AsJSValue},
    ffi,
    marker::Covariant,
    value::Value,
};
use std::{ffi::c_void, fmt, marker::PhantomData, ptr::NonNull};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Runtime<'q>(NonNull<ffi::JSRuntime>, Covariant<'q>);

impl<'q> Runtime<'q> {
    // lifecycle

    #[inline]
    pub unsafe fn from_ptr(ptr: *mut ffi::JSRuntime) -> Runtime<'q> {
        Runtime(NonNull::new(ptr).unwrap(), PhantomData)
    }

    #[inline]
    pub fn new() -> Runtime<'q> {
        unsafe { Self::from_ptr(ffi::JS_NewRuntime()) }
    }

    #[inline]
    pub unsafe fn free(this: Self) {
        ffi::JS_FreeRuntime(this.0.as_ptr());
    }

    // basic

    #[inline]
    pub fn opaque(self) -> *mut c_void {
        unsafe { ffi::JS_GetRuntimeOpaque(self.0.as_ptr()) }
    }

    #[inline]
    pub fn set_opaque(self, opaque: *mut c_void) {
        unsafe { ffi::JS_SetRuntimeOpaque(self.0.as_ptr(), opaque) }
    }

    #[inline]
    pub fn run_gc(self) {
        unsafe { ffi::JS_RunGC(self.0.as_ptr()) }
    }

    #[inline]
    pub fn is_live_object(self, value: Value) -> bool {
        unsafe { ffi::JS_IsLiveObject(self.0.as_ptr(), value.as_js_value()) != 0 }
    }

    // class

    #[inline]
    pub fn new_class(self, id: ClassId, class_def: &ClassDef) {
        unsafe {
            let result = ffi::JS_NewClass(self.0.as_ptr(), ClassId::raw(id), &class_def.c_def());
            assert_eq!(0, result)
        }
    }

    #[inline]
    pub fn id_registered_class(self, id: ClassId) -> bool {
        unsafe { ffi::JS_IsRegisteredClass(self.0.as_ptr(), ClassId::raw(id)) != 0 }
    }

    // value

    #[inline]
    pub unsafe fn free_value(self, value: Value<'q>) {
        ffi::JS_FreeValueRT(self.0.as_ptr(), value.as_js_value());
    }

    #[inline]
    pub fn dup_value(self, value: Value<'q>) -> Value<'q> {
        unsafe {
            let res = ffi::JS_DupValueRT(self.0.as_ptr(), value.as_js_value());
            Value::from_raw_with_runtime(res, self)
        }
    }
}

impl fmt::Debug for Runtime<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.write_str(format!("Runtime({:p})", self.0).as_str())
    }
}

impl AsJSRuntimePointer for Runtime<'_> {
    fn as_ptr(&self) -> *mut ffi::JSRuntime {
        self.0.as_ptr()
    }
}
