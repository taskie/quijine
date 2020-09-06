use crate::core::{
    class::{ClassDef, ClassId},
    conversion::AsJSValue,
    ffi,
    marker::Covariant,
    value::Value,
};
use lazy_static::lazy_static;
use std::{ffi::c_void, fmt, marker::PhantomData, ptr::NonNull, sync::Mutex};

lazy_static! {
    static ref NEW_CLASS_ID_LOCK: Mutex<()> = Mutex::new(());
}

pub trait AsJSRuntimePointer {
    fn as_ptr(&self) -> *mut ffi::JSRuntime;
}

impl AsJSRuntimePointer for *mut ffi::JSRuntime {
    fn as_ptr(&self) -> *mut ffi::JSRuntime {
        *self
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Runtime<'q>(NonNull<ffi::JSRuntime>, Covariant<'q>);

impl<'q> Runtime<'q> {
    #[inline]
    pub unsafe fn from_ptr(ptr: *mut ffi::JSRuntime) -> Runtime<'q> {
        Runtime(NonNull::new(ptr).unwrap(), PhantomData)
    }

    #[inline]
    pub(crate) unsafe fn from_ptr_unchecked(ptr: *mut ffi::JSRuntime) -> Runtime<'q> {
        Runtime(NonNull::new_unchecked(ptr), PhantomData)
    }

    #[inline]
    pub fn new() -> Runtime<'q> {
        unsafe { Self::from_ptr(ffi::JS_NewRuntime()) }
    }

    #[inline]
    pub unsafe fn free(this: Self) {
        ffi::JS_FreeRuntime(this.0.as_ptr());
    }

    #[inline]
    pub unsafe fn raw(this: Self) -> *mut ffi::JSRuntime {
        this.0.as_ptr()
    }

    #[inline]
    pub unsafe fn free_value(self, value: Value<'q>) {
        ffi::JS_FreeValueRT(self.as_ptr(), value.as_js_value());
    }

    #[inline]
    pub fn dup_value(self, value: Value<'q>) {
        unsafe {
            ffi::JS_DupValueRT(self.as_ptr(), value.as_js_value());
        }
    }

    #[inline]
    pub fn run_gc(self) {
        unsafe { ffi::JS_RunGC(self.as_ptr()) }
    }

    #[inline]
    pub fn is_live_object(self, value: Value) -> bool {
        unsafe { ffi::JS_IsLiveObject(self.as_ptr(), value.as_js_value()) != 0 }
    }

    #[inline]
    pub fn new_class_id(self) -> ClassId {
        let mut id = 0;
        {
            // JS_NewClassID is not thread-safe...
            let _ = NEW_CLASS_ID_LOCK.lock().unwrap();
            unsafe {
                ffi::JS_NewClassID(&mut id);
            };
        }
        ClassId::new(id)
    }

    #[inline]
    pub fn new_class(self, id: ClassId, class_def: &ClassDef) {
        unsafe {
            let result = ffi::JS_NewClass(self.as_ptr(), ClassId::raw(id), &class_def.c_def());
            assert_eq!(0, result)
        }
    }

    #[inline]
    pub fn opaque(self) -> *mut c_void {
        unsafe { ffi::JS_GetRuntimeOpaque(self.as_ptr()) }
    }

    #[inline]
    pub fn set_opaque(self, opaque: *mut c_void) {
        unsafe { ffi::JS_SetRuntimeOpaque(self.as_ptr(), opaque) }
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
