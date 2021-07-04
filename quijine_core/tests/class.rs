use std::{ffi::CString, ptr::null_mut};

use quijine_core::{raw, AsJsValue, ClassDef, ClassId, Context, Runtime};

#[test]
fn test() {
    let rt = Runtime::new();
    let ctx = Context::new(rt);
    // Define Class
    let class_name = CString::new("Test").unwrap();
    let test_class_def = unsafe {
        ClassDef::from_raw(raw::JSClassDef {
            class_name: class_name.as_ptr(),
            finalizer: None,
            gc_mark: None,
            call: None,
            exotic: null_mut(),
        })
    };
    let test_class_id = ClassId::generate();
    rt.new_class(test_class_id, &test_class_def);
    let test_proto = ctx.new_object();
    ctx.set_class_proto(test_class_id, test_proto);
    // Use Class
    let obj = ctx.new_object_class(test_class_id);
    let obj_proto = obj.prototype(ctx);
    assert!(obj_proto.is_object());
    assert!(!obj_proto.is_null());
    unsafe {
        assert_eq!(obj_proto.as_js_value().u.ptr, test_proto.as_js_value().u.ptr);
    }
    unsafe {
        ctx.free_value(obj_proto);
        ctx.free_value(obj);
        Context::free(ctx);
        Runtime::free(rt);
    }
}

#[test]
fn test_another_context() {
    let rt = Runtime::new();
    let ctx = Context::new(rt);
    // Define Class
    let class_name = CString::new("Test").unwrap();
    let test_class_def = unsafe {
        ClassDef::from_raw(raw::JSClassDef {
            class_name: class_name.as_ptr(),
            finalizer: None,
            gc_mark: None,
            call: None,
            exotic: null_mut(),
        })
    };
    let test_class_id = ClassId::generate();
    rt.new_class(test_class_id, &test_class_def);
    let test_proto = ctx.new_object();
    ctx.set_class_proto(test_class_id, test_proto);
    unsafe {
        Context::free(ctx);
    }
    // Use Class
    let ctx = Context::new(rt);
    let obj = ctx.new_object_class(test_class_id);
    let obj_proto = obj.prototype(ctx);
    // the prototype is null...
    assert!(obj_proto.is_null());
    unsafe {
        ctx.free_value(obj_proto);
        ctx.free_value(obj);
        Context::free(ctx);
        Runtime::free(rt);
    }
}
