use crate::{convert::AsJsClassId, ffi};
use lazy_static::lazy_static;
use std::sync::Mutex;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(transparent)]
pub struct ClassId(u32);

impl ClassId {
    #[inline]
    pub fn from_raw(id: ffi::JSClassID) -> ClassId {
        ClassId(id)
    }

    #[inline]
    pub fn none() -> ClassId {
        ClassId::from_raw(0)
    }

    #[inline]
    pub fn generate() -> ClassId {
        ClassId::none().new_class_id()
    }

    #[inline]
    pub(crate) fn new_class_id(&mut self) -> ClassId {
        let res = {
            // JS_NewClassID is not thread-safe...
            let _lock = NEW_CLASS_ID_LOCK.lock().unwrap();
            unsafe { ffi::JS_NewClassID(&mut self.0) }
        };
        assert_eq!(res, self.0);
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

#[derive(Clone, Debug)]
pub struct ClassDef(ffi::JSClassDef);

impl ClassDef {
    /// # Safety
    /// class must have the same lifetime as a runtime.
    #[inline]
    pub unsafe fn from_raw(c: ffi::JSClassDef) -> ClassDef {
        ClassDef(c)
    }
}

impl AsRef<ffi::JSClassDef> for ClassDef {
    fn as_ref(&self) -> &ffi::JSClassDef {
        &self.0
    }
}
