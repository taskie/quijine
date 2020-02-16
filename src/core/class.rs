use std::{ffi::CString, ptr::null_mut};

use crate::core::ffi;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClassID(u32);

impl ClassID {
    #[inline]
    pub fn new(id: u32) -> ClassID {
        ClassID(id)
    }

    #[inline]
    pub fn raw(this: Self) -> u32 {
        this.0
    }
}

pub struct ClassDef {
    pub class_name: String,
    pub finalizer: ffi::JSClassFinalizer,
    pub gc_mark: ffi::JSClassGCMark,
    #[doc(hidden)]
    pub call: ffi::JSClassCall,
    #[doc(hidden)]
    pub exotic: *mut ffi::JSClassExoticMethods,
}

impl ClassDef {
    pub(crate) fn c_def(&self) -> ffi::JSClassDef {
        let c_str = CString::new(self.class_name.as_str()).unwrap();
        ffi::JSClassDef {
            class_name: c_str.as_ptr(),
            finalizer: self.finalizer,
            gc_mark: self.gc_mark,
            call: self.call,
            exotic: self.exotic,
        }
    }
}

impl Default for ClassDef {
    fn default() -> Self {
        ClassDef {
            class_name: String::default(),
            finalizer: None,
            gc_mark: None,
            call: None,
            exotic: null_mut(),
        }
    }
}
