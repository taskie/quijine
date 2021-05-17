use crate::{conversion::AsJsClassId, ffi, Runtime, Value};
use lazy_static::lazy_static;
use std::{ffi::CString, ptr::null_mut, sync::Mutex};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(transparent)]
pub struct ClassId(u32);

impl ClassId {
    #[inline]
    pub fn from_raw(id: ffi::JSClassID) -> ClassId {
        ClassId(id)
    }

    pub fn none() -> ClassId {
        ClassId::from_raw(0)
    }

    pub fn generate() -> ClassId {
        ClassId::none().new_class_id()
    }

    pub(crate) fn new_class_id(self) -> ClassId {
        let mut before = self.0;
        let res = {
            // JS_NewClassID is not thread-safe...
            let _lock = NEW_CLASS_ID_LOCK.lock().unwrap();
            unsafe { ffi::JS_NewClassID(&mut before) }
        };
        ClassId::from_raw(res)
    }
}

impl<'q> AsJsClassId<'q> for ClassId {
    #[inline]
    fn as_js_class_id(&self) -> ffi::JSClassID {
        self.0
    }
}

lazy_static! {
    static ref NEW_CLASS_ID_LOCK: Mutex<()> = Mutex::new(());
}

type ClassFinalizer<'q> = fn(Runtime<'q>, Value<'q>) -> ();

#[derive(Debug)]
pub struct ClassDef {
    pub class_name: CString,
    pub finalizer: ffi::JSClassFinalizer,
    pub gc_mark: ffi::JSClassGCMark,
    #[doc(hidden)]
    pub call: ffi::JSClassCall,
    #[doc(hidden)]
    pub exotic: *mut ffi::JSClassExoticMethods,
}

impl ClassDef {
    pub(crate) fn c_def(&self) -> ffi::JSClassDef {
        ffi::JSClassDef {
            class_name: self.class_name.as_ptr(),
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
            class_name: CString::default(),
            finalizer: None,
            gc_mark: None,
            call: None,
            exotic: null_mut(),
        }
    }
}
