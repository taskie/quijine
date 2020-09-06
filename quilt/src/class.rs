use crate::{conversion::AsJsClassId, ffi};
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
            let _ = NEW_CLASS_ID_LOCK.lock().unwrap();
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

#[cfg(test)]
mod tests {
    use crate::{js_c_function, js_class_finalizer, ClassDef, ClassId, Context, EvalFlags, Runtime, Value};
    use std::{cell::RefCell, ffi::c_void, ptr::null_mut};

    struct S1 {
        name: String,
        pos: (i32, i32),
    }

    thread_local! {
        static S1_CLASS_ID: RefCell<ClassId> = RefCell::new(ClassId::none());
    }

    unsafe fn new_s1<'q, 'a>(ctx: Context<'q>, _this_val: Value<'q>, _values: &'a [Value<'q>]) -> Value<'q> {
        let obj = S1_CLASS_ID.with(|id| ctx.new_object_class(*id.borrow()));
        let s1 = Box::new(S1 {
            name: "Hello!".to_owned(),
            pos: (3, 4),
        });
        let s1_ptr = Box::into_raw(s1) as *mut c_void;
        obj.set_opaque(s1_ptr);
        obj
    }

    unsafe fn finalize_s1(_rt: Runtime, val: Value) {
        let s1_ptr = S1_CLASS_ID.with(|id| val.opaque(*id.borrow()) as *mut S1);
        Box::from_raw(s1_ptr);
    }

    #[test]
    fn test() {
        let rt = Runtime::new();
        let ctx = Context::new(rt);
        S1_CLASS_ID.with(|id| {
            *id.borrow_mut() = ClassId::generate();
            rt.new_class(
                *id.borrow(),
                &ClassDef {
                    class_name: "S1".to_owned(),
                    finalizer: js_class_finalizer!(finalize_s1),
                    ..Default::default()
                },
            );
            let s1_proto = ctx.new_object();
            ctx.set_class_proto(*id.borrow(), s1_proto);
        });
        let global = ctx.global_object();
        global.set_property_str(ctx, "S1", unsafe {
            ctx.new_c_function(js_c_function!(new_s1), "S1", 0)
        });
        let ret = ctx.eval(
            "const s1 = S1(); const s2 = s1; const s3 = s2; s3",
            "<input>",
            EvalFlags::TYPE_GLOBAL,
        );
        assert_eq!(false, ctx.exception().is_exception(), "no exception");
        S1_CLASS_ID.with(|id| {
            assert_ne!(null_mut(), ret.opaque(*id.borrow()), "valid class_id");
        });
        unsafe {
            ctx.free_value(ret);
            ctx.free_value(global);
            Context::free(ctx);
            Runtime::free(rt);
        }
    }
}
